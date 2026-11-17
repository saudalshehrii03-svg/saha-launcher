import EmptyState from "../components/EmptyState";

export default function Library() {
  return (
    <section>
      <h1 className="page-title">المكتبة</h1>
      <EmptyState title="المكتبة فارغة" text="سيظهر هنا كل ألعابك من كل المنصات والحسابات بعد بناء الفحص (المرحلة ١)." />
    </section>
  );
}
