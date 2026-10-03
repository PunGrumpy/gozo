/* overflow-x-clip, not hidden: hidden turns the wrapper into a scroll container and breaks every view() timeline */
const Layout = ({ children }: LayoutProps<"/">) => (
  <div className="bg-background-100 text-gray-1000 relative isolate overflow-x-clip lowercase [--gutter:24px] md:[--gutter:32px]">
    {children}
  </div>
);

export default Layout;
