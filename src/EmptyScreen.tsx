import { t } from "./i18n";

export default function EmptyScreen({ onOpen }: { onOpen: () => void }) {
  return (
    <div className="empty">
      <button onClick={onOpen}>{t.open}</button>
      <p>{t.dropHint}</p>
    </div>
  );
}
