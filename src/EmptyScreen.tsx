export default function EmptyScreen({ onOpen }: { onOpen: () => void }) {
  return (
    <div className="empty">
      <button onClick={onOpen}>Открыть…</button>
      <p>или перетащите файл в окно</p>
    </div>
  );
}
