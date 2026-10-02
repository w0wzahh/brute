import TopNav from "@/components/TopNav";
import Footer from "@/components/Footer";

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <>
      <TopNav />
      {children}
      <Footer />
    </>
  );
}
