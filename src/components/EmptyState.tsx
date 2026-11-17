export default function EmptyState({ title, text }: { title: string; text: string }) {
  return (
    <div className="empty glass">
      <h2>{title}</h2>
      <p>{text}</p>
    </div>
  );
}
