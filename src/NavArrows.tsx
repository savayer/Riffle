import { useEffect, useState } from "react";
import { t } from "./i18n";

const HIDE_AFTER_MS = 2000;

type Props = { canPrev: boolean; canNext: boolean; onPrev: () => void; onNext: () => void };

function Chevron({ d }: { d: string }) {
  return (
    <svg viewBox="0 0 24 24" width="24" height="24">
      <path d={d} fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

/** Slider-style ‹ › buttons: shown on mouse move, faded out after a short idle. */
export default function NavArrows({ canPrev, canNext, onPrev, onNext }: Props) {
  const [visible, setVisible] = useState(true);

  useEffect(() => {
    let timer = setTimeout(() => setVisible(false), HIDE_AFTER_MS);
    const onMove = (e: MouseEvent) => {
      setVisible(true);
      clearTimeout(timer);
      // keep them while the cursor rests on a button
      if (!(e.target as Element).closest?.(".nav")) {
        timer = setTimeout(() => setVisible(false), HIDE_AFTER_MS);
      }
    };
    window.addEventListener("mousemove", onMove);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("mousemove", onMove);
    };
  }, []);

  const cls = (side: string) => `nav ${side}${visible ? "" : " hidden"}`;
  // no focus on click: Space must keep toggling the video, not "press" the button
  const noFocus = (e: React.MouseEvent) => e.preventDefault();

  return (
    <>
      {canPrev && (
        <button className={cls("prev")} tabIndex={-1} onMouseDown={noFocus} onClick={onPrev} aria-label={t.prev}>
          <Chevron d="M15 6l-6 6 6 6" />
        </button>
      )}
      {canNext && (
        <button className={cls("next")} tabIndex={-1} onMouseDown={noFocus} onClick={onNext} aria-label={t.next}>
          <Chevron d="M9 6l6 6-6 6" />
        </button>
      )}
    </>
  );
}
