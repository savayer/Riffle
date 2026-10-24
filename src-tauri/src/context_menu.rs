//! Tunes WebKit's own context menu to fit a local viewer, like Preview's: drops the
//! browser leftovers (Open Image in New Window, Download Image, Reload...) and adds
//! "Show in Finder". JS can only replace that menu, not edit it, but AppKit calls
//! `willOpenMenu:withEvent:` on the view right before showing it, so that method is
//! added to wry's WKWebView subclass at runtime.

use std::ffi::{CStr, c_char, c_void};
use std::sync::Mutex;
use std::sync::atomic::{AtomicPtr, Ordering};

use objc2::ffi::class_addMethod;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject, Imp, NSObject, Sel};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{NSEvent, NSImage, NSMenu, NSMenuItem, NSUserInterfaceItemIdentification};
use objc2_foundation::{NSBundle, NSString, ns_string};

/// File on screen, kept in sync by the frontend; `None` = empty screen, no Finder item.
static CURRENT: Mutex<Option<String>> = Mutex::new(None);
// NSMenuItem.target is weak: the target lives for the whole process
static TARGET: AtomicPtr<AnyObject> = AtomicPtr::new(std::ptr::null_mut());

pub fn set_current(path: Option<String>) {
    *CURRENT.lock().unwrap() = path;
}

// WebKit's exported item identifiers (SPI `_WKMenuItemIdentifier*`): stable across
// languages, unlike titles. Looked up at runtime so an older macOS lacking one still launches.
const BROWSER_ITEMS: [&CStr; 12] = [
    c"_WKMenuItemIdentifierOpenImageInNewWindow",
    c"_WKMenuItemIdentifierDownloadImage",
    c"_WKMenuItemIdentifierOpenMediaInNewWindow",
    c"_WKMenuItemIdentifierDownloadMedia",
    c"_WKMenuItemIdentifierCopyMediaLink",
    c"_WKMenuItemIdentifierOpenFrameInNewWindow",
    c"_WKMenuItemIdentifierLookUp",
    c"_WKMenuItemIdentifierSearchWeb",
    c"_WKMenuItemIdentifierReload",
    c"_WKMenuItemIdentifierGoBack",
    c"_WKMenuItemIdentifierGoForward",
    c"_WKMenuItemIdentifierInspectElement",
];

unsafe extern "C" {
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}
const RTLD_DEFAULT: *mut c_void = -2isize as *mut c_void;

fn is_browser_item(item: &NSMenuItem) -> bool {
    let Some(id) = item.identifier() else {
        return false; // e.g. Play / Mute / Loop carry no identifier: keep
    };
    BROWSER_ITEMS.iter().any(|name| {
        // the symbol is an `NSString *` variable
        let var = unsafe { dlsym(RTLD_DEFAULT, name.as_ptr()) } as *const &NSString;
        !var.is_null() && unsafe { **var == *id }
    })
}

/// Removes leading, trailing and doubled separators left after dropping items.
fn tidy_separators(menu: &NSMenu) {
    let mut i = 0;
    let mut after_separator = true; // treats the top as a separator
    while i < menu.numberOfItems() {
        let separator = menu.itemAtIndex(i).is_some_and(|it| it.isSeparatorItem());
        if separator && after_separator {
            menu.removeItemAtIndex(i);
            continue;
        }
        after_separator = separator;
        i += 1;
    }
    if after_separator && menu.numberOfItems() > 0 {
        menu.removeItemAtIndex(menu.numberOfItems() - 1);
    }
}

define_class!(
    // own target instead of the webview: WebKit's item validation would disable
    // a selector it doesn't know
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "RiffleMenuTarget"]
    struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(revealInFinder:))]
        fn reveal_in_finder(&self, _sender: Option<&AnyObject>) {
            if let Some(path) = CURRENT.lock().unwrap().clone() {
                std::thread::spawn(move || crate::reveal(&path));
            }
        }
    }
);

/// Same language pick AppKit/WebKit make for their own items (Info.plist lists en, ru).
fn reveal_title() -> &'static NSString {
    let lang = NSBundle::mainBundle()
        .preferredLocalizations()
        .firstObject();
    if lang.is_some_and(|l| l.to_string().starts_with("ru")) {
        ns_string!("Показать в Finder")
    } else {
        ns_string!("Show in Finder")
    }
}

unsafe extern "C-unwind" fn will_open_menu(
    this: &AnyObject,
    _cmd: Sel,
    menu: &NSMenu,
    event: &NSEvent,
) {
    // WebKit fills in its items first. Call WKWebView's implementation explicitly:
    // the runtime class may be a KVO subclass, so "superclass of my class" could recurse.
    if let Some(wk) = AnyClass::get(c"WKWebView") {
        let _: () = unsafe { msg_send![super(this, wk), willOpenMenu: menu, withEvent: event] };
    }
    for i in (0..menu.numberOfItems()).rev() {
        if menu.itemAtIndex(i).is_some_and(|it| is_browser_item(&it)) {
            menu.removeItemAtIndex(i);
        }
    }
    tidy_separators(menu);

    if CURRENT.lock().unwrap().is_none() {
        return;
    }
    let mtm = MainThreadMarker::new().expect("menus open on the main thread");
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            reveal_title(),
            Some(sel!(revealInFinder:)),
            ns_string!("r"), // shown as ⌘R; the shortcut itself is handled in JS
        )
    };
    unsafe { item.setTarget(TARGET.load(Ordering::Relaxed).as_ref()) };
    // icon like WebKit's own items; without it the title sits out of line
    item.setImage(
        NSImage::imageWithSystemSymbolName_accessibilityDescription(ns_string!("folder"), None)
            .as_deref(),
    );
    if menu.numberOfItems() > 0 {
        menu.insertItem_atIndex(&NSMenuItem::separatorItem(mtm), 0);
    }
    menu.insertItem_atIndex(&item, 0);
}

/// `webview` is the WKWebView pointer from `PlatformWebview::inner()`; call on the main thread.
pub fn install(webview: *mut std::ffi::c_void) {
    let (Some(mtm), Some(wk)) = (MainThreadMarker::new(), AnyClass::get(c"WKWebView")) else {
        return;
    };
    // wry's subclass = the direct child of WKWebView. Found by walking up from the
    // instance: its generated class name isn't stable, and KVO may sit on top of it.
    let Some(webview) = (unsafe { (webview as *const AnyObject).as_ref() }) else {
        return;
    };
    let mut class = webview.class();
    while let Some(parent) = class.superclass() {
        if parent == wk {
            break;
        }
        class = parent;
    }
    if class.superclass() != Some(wk) {
        return; // never patch WKWebView itself: the super call would recurse
    }
    let target: Retained<MenuTarget> = unsafe { msg_send![MenuTarget::alloc(mtm), init] };
    TARGET.store(Retained::into_raw(target).cast(), Ordering::Relaxed);
    let imp = unsafe {
        std::mem::transmute::<unsafe extern "C-unwind" fn(&AnyObject, Sel, &NSMenu, &NSEvent), Imp>(
            will_open_menu,
        )
    };
    // wry doesn't implement this method, so adding it overrides WKWebView's for every webview
    unsafe {
        class_addMethod(
            class as *const AnyClass as *mut AnyClass,
            sel!(willOpenMenu:withEvent:),
            imp,
            c"v@:@@".as_ptr(),
        );
    }
}
