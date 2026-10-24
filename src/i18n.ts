// UI follows the system language: Russian on a Russian macOS, English otherwise
const ru = navigator.language.toLowerCase().startsWith("ru");
document.documentElement.lang = ru ? "ru" : "en";

export const t = ru
  ? {
      open: "Открыть…",
      dropHint: "или перетащите файл в окно",
      media: "Медиа",
      prev: "Предыдущий",
      next: "Следующий",
    }
  : {
      open: "Open…",
      dropHint: "or drop a file into the window",
      media: "Media",
      prev: "Previous",
      next: "Next",
    };
