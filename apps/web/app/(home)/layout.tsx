import "./kit.generated.css";
import "./landing.css";

const Layout = ({ children }: LayoutProps<"/">) => (
  <div className="landing">{children}</div>
);

export default Layout;
