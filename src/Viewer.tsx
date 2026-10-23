import { useEffect, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { fileName, isVideo } from "./media";

type Preload = { img: HTMLImageElement; ready: Promise<boolean> };
type Shown = { path: string; ok: boolean };

function preload(path: string): Preload {
  const img = new Image();
  img.src = convertFileSrc(path);
  return { img, ready: img.decode().then(() => true, () => false) };
}

function Placeholder({ path }: { path: string }) {
  return (
    <div className="broken">
      <div className="broken-icon">⚠︎</div>
      {fileName(path)}
    </div>
  );
}

export default function Viewer({ files, index }: { files: string[]; index: number }) {
  const path = files[index];
  const video = isVideo(path);
  // the image on screen; replaced only once the next one has decoded, so no flicker
  const [shown, setShown] = useState<Shown | null>(null);
  const [badVideo, setBadVideo] = useState<string | null>(null);
  const cache = useRef(new Map<string, Preload>());
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    const keep = new Map<string, Preload>();
    for (let i = Math.max(0, index - 2); i <= Math.min(files.length - 1, index + 2); i++) {
      const p = files[i];
      if (!isVideo(p)) keep.set(p, cache.current.get(p) ?? preload(p));
    }
    cache.current = keep;
    if (video) {
      setShown(null);
      return;
    }
    let stale = false;
    keep.get(path)!.ready.then((ok) => !stale && setShown({ path, ok }));
    return () => {
      stale = true;
    };
  }, [files, index]);

  useEffect(() => {
    if (!video) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== " ") return;
      e.preventDefault();
      e.stopPropagation(); // native controls must not toggle a second time
      const v = videoRef.current;
      if (v) v.paused ? v.play() : v.pause();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [video]);

  let content = null;
  if (video) {
    content =
      badVideo === path ? (
        <Placeholder path={path} />
      ) : (
        <video
          key={path}
          ref={videoRef}
          src={convertFileSrc(path)}
          autoPlay
          controls
          tabIndex={-1}
          onMouseUp={(e) => e.currentTarget.blur()}
          onError={() => setBadVideo(path)}
        />
      );
  } else if (shown) {
    content = shown.ok ? (
      <img src={convertFileSrc(shown.path)} alt="" draggable={false} />
    ) : (
      <Placeholder path={shown.path} />
    );
  }

  // single wrapper: a future zoom is a transform on .stage
  return <div className="stage">{content}</div>;
}
