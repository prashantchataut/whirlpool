import { Charging } from "./components/Charging";
import { SettingsLab } from "./components/SettingsLab";
import { DockDemo, Footer, Hero, Install, Nav, Section, Stats, Trust, Wallpapers, WhatsNew } from "./components/Sections";

export default function App() {
  return (
    <div className="relative overflow-x-clip">
      <a
        href="#dock"
        className="fixed left-4 top-4 z-[60] -translate-y-20 rounded-lg bg-teal px-3 py-2 text-sm font-semibold text-ink focus:translate-y-0"
      >
        Skip to content
      </a>
      <Nav />
      <main>
        <Hero />
        <Stats />
        <Section
          id="dock"
          eyebrow="The dock"
          title="Everything in reach, nothing in the way."
          lead="Apps, files and open windows on one glass shelf. Put it on any edge and keep only what you use."
        >
          <DockDemo />
        </Section>
        <Section
          id="customize"
          eyebrow="Settings"
          title="Every edge is adjustable."
          lead="An interactive model of the Settings page, built for the web so you can try the options. The window itself is in the screenshots above."
        >
          <SettingsLab />
        </Section>
        <Section
          id="charging"
          eyebrow="Charging"
          title="Six ways to say 'plugged in'."
          lead="Green when the battery is healthy, yellow in the middle, orange when it is low. Pick one, or turn them off."
        >
          <Charging />
        </Section>
        <Section
          id="wallpapers"
          eyebrow="Wallpapers"
          title="Fourteen photographs, stored in the app."
        >
          <Wallpapers />
        </Section>
        <Section
          id="trust"
          eyebrow="Trust"
          title="Easy to try. Easy to undo."
          lead="Unsigned software is a fair thing to be careful about. Here is how to check this one, and how to leave."
        >
          <Trust />
        </Section>
        <Section id="changes" eyebrow="This release" title="What is new in 4.2.">
          <WhatsNew />
        </Section>
        <Install />
      </main>
      <Footer />
    </div>
  );
}
