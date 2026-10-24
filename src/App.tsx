import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import EmptyScreen from "./EmptyScreen";
import NavArrows from "./NavArrows";
import Viewer from "./Viewer";
import { t } from "./i18n";
import { fileName, MEDIA_EXTS } from "./media";
import { navigate } from "./nav";
import { createSwipe } from "./swipe";
import "./App.css";

type MediaList = { files: string[]; index: number };

export default function App() {
  // plain mutable list: a future "move to trash" is splice + clamp index
  const [list, setList] = useState<MediaList>({ files: [], index: 0 });
  const listRef = useRef(list);
  listRef.current = list;
  const request = useRef(0);
  const { files, index } = list;
  const current = files[index];

  async function openPath(path: string) {
    const id = ++request.current;
    let next: MediaList;
    try {
      next = await invoke<MediaList>("list_media", { path });
    } catch {
      next = { files: [path], index: 0 }; // Viewer shows the placeholder
    }
    if (id === request.current) setList(next); // drop answers to superseded opens
  }

  function go(key: string): boolean {
    const { files, index } = listRef.current;
    const i = navigate(index, files.length, key);
    if (i === null) return false;
    if (i !== index) setList({ files, index: i });
    return true;
  }

  function reveal() {
    const { files, index } = listRef.current;
    if (files[index]) invoke("reveal_in_finder", { path: files[index] });
  }

  async function pickFile() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: t.media, extensions: MEDIA_EXTS }],
    });
    if (path) openPath(path);
  }

  useEffect(() => {
    const unlisten = [
      listen<string>("open-file", (e) => openPath(e.payload)),
      getCurrentWebview().onDragDropEvent((e) => {
        if (e.payload.type === "drop" && e.payload.paths[0]) openPath(e.payload.paths[0]);
      }),
    ];
    // subscribe first, then ask: a Finder open arriving in between is not lost
    Promise.all(unlisten)
      .then(() => invoke<string | null>("get_initial_file"))
      .then((path) => {
        if (path) openPath(path);
      });

    const onKey = (e: KeyboardEvent) => {
      if (e.metaKey && e.key.toLowerCase() === "r") {
        e.preventDefault();
        reveal();
      } else if (go(e.key)) {
        e.preventDefault();
        e.stopPropagation(); // keep arrows away from <video> seeking
      }
    };
    const onWheel = createSwipe((dir) => go(dir === 1 ? "ArrowRight" : "ArrowLeft"));
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("wheel", onWheel);
    return () => {
      unlisten.forEach((u) => u.then((f) => f()));
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("wheel", onWheel);
    };
  }, []);

  useEffect(() => {
    // the native context menu offers "Show in Finder" only while a file is shown
    invoke("set_current_file", { path: current ?? null });
  }, [current]);

  useEffect(() => {
    getCurrentWindow().setTitle(
      current ? `${fileName(current)} — ${index + 1} / ${files.length}` : "Riffle",
    );
  }, [current, index, files.length]);

  if (!current) return <EmptyScreen onOpen={pickFile} />;
  return (
    <>
      <Viewer files={files} index={index} />
      <NavArrows
        canPrev={index > 0}
        canNext={index < files.length - 1}
        onPrev={() => go("ArrowLeft")}
        onNext={() => go("ArrowRight")}
      />
    </>
  );
}
