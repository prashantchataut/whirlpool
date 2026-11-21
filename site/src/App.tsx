import { BackdropVortex, Behavior, DepthGauge, Footer, Header, Hero, Release, Screens, Specs } from "./components/Sections";
import { useReveal } from "./lib/hooks";

export default function App() {
  useReveal();
  return (
    <div className="min-h-screen">
      <Header />
      <DepthGauge />
      <main>
        <Hero />
        <div className="water-column relative">
          <BackdropVortex />
          <Screens />
          <Behavior />
          <Release />
          <Specs />
          <Footer />
        </div>
      </main>
    </div>
  );
}
