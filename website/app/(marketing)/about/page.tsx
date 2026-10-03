import type { Metadata } from "next";
import Window from "@/components/Window";

export const metadata: Metadata = { title: "About the Developer" };

const PROJECTS = [
  {
    icon: "fas fa-globe",
    title: "Web Development Portfolio",
    desc: "Websites and web apps built over the years — from scrapers to full-stack apps.",
    tags: ["HTML5", "CSS3", "JavaScript", "React"],
  },
  {
    icon: "fas fa-shield-halved",
    title: "Security Research Tools",
    desc: "Educational pentesting tools for analyzing vulnerabilities and understanding protection mechanisms.",
    tags: ["Python", "C++", "Assembly"],
  },
  {
    icon: "fas fa-server",
    title: "Full-Stack Applications",
    desc: "Apps with real front-ends and back-end services — auth, databases, API integration.",
    tags: ["Node.js", "Express", "MongoDB", "React"],
  },
];

const SKILLS = [
  {
    icon: "fas fa-code",
    title: "Languages",
    tags: ["C/C++", "Python", "JavaScript", "Rust", "Java", "Assembly", "Brute"],
  },
  {
    icon: "fas fa-layer-group",
    title: "Web",
    tags: ["HTML5", "CSS3", "React", "Node.js", "Express", "MongoDB", "SQL"],
  },
  {
    icon: "fas fa-toolbox",
    title: "Tools",
    tags: ["Git", "Docker", "Linux", "VS Code", "AWS", "LLVM"],
  },
];

export default function About() {
  return (
    <section className="documentation-content">
      <div className="container">
        <div className="documentation-header">
          <h1>About the Developer</h1>
          <p>The mind behind Brute.</p>
        </div>

        <Window id="bio" title="creator.exe">
          <div className="about-card">
            <div className="profile-img">
              <i className="fas fa-terminal" />
            </div>
            <div className="profile-info">
              <h3>w0wzahh</h3>
              <p>
                <i className="fas fa-map-marker-alt" /> Turkish, based in Pécs,
                Hungary
              </p>
              <p>
                <i className="fas fa-code" /> Developer of Brute
              </p>
              <div className="profile-links">
                <a
                  href="https://github.com/w0wzahh/brute"
                  target="_blank"
                  rel="noreferrer"
                  className="gh-icon"
                  aria-label="Brute on GitHub"
                  title="Brute on GitHub"
                >
                  <i className="fab fa-github" />
                </a>
                <a
                  href="https://w0wzahh.link"
                  target="_blank"
                  rel="noreferrer"
                  className="gh-icon"
                  aria-label="w0wzahh portfolio — w0wzahh.link"
                  title="w0wzahh.link"
                >
                  <i className="fas fa-globe" />
                </a>
              </div>
            </div>
          </div>
          <div className="about-content">
            <p>
              I&apos;m a Turkish programmer living in Pécs, Hungary — coding
              since I was 10. I got hooked through security research and systems
              programming, then moved into web dev and full-stack work. I
              started building Brute when I was 17.
            </p>
            <p>
              <strong>Brute</strong> is the culmination of that: a language that
              takes Rust&apos;s safety, Python&apos;s readability, and C&apos;s
              speed — without making you pick two. It started as a personal
              challenge and turned into a real project I&apos;m actively
              building.
            </p>
            <p>
              My philosophy is simple: performance, safety, and usability
              shouldn&apos;t be a trade-off. If you want to contribute or have
              ideas, the repo&apos;s right there.
            </p>
          </div>
        </Window>

        <Window id="projects" title="other_projects.exe">
          <h2>Other Projects</h2>
          <div className="project-grid">
            {PROJECTS.map((p) => (
              <div key={p.title} className="project-card">
                <span className="project-ico">
                  <i className={p.icon} />
                </span>
                <h3>{p.title}</h3>
                <p>{p.desc}</p>
                <div className="tech-tags">
                  {p.tags.map((t) => (
                    <span key={t} className="tag">
                      {t}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Window>

        <Window id="skills" title="skills.exe">
          <h2>Technical Skills</h2>
          <div className="skills-container">
            {SKILLS.map((s) => (
              <div key={s.title} className="skill-category">
                <h3>
                  <i className={s.icon} /> {s.title}
                </h3>
                <div className="skill-tags">
                  {s.tags.map((t) => (
                    <span key={t} className="tag">
                      {t}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </Window>
      </div>
    </section>
  );
}
