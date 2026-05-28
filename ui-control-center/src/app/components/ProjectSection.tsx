"use client";

import { type ReactNode } from "react";

interface ProjectSectionProps {
  id: string;
  number: string;
  title: string;
  subtitle: string;
  description: string;
  highlights: string[];
  tags: string[];
  githubUrl?: string;
  demo: ReactNode;
}

export default function ProjectSection({
  id,
  number,
  title,
  subtitle,
  description,
  highlights,
  tags,
  githubUrl,
  demo,
}: ProjectSectionProps) {
  return (
    <section id={id} className="section">
      <div className="section-heading">{number} — Project</div>
      <h2 className="section-title">{title}</h2>
      <p className="text-text-soft text-base mb-4 max-w-2xl">{subtitle}</p>

      <div className="flex flex-col lg:flex-row gap-8 mt-8">
        <div className="lg:w-[380px] shrink-0 space-y-6">
          <p className="text-text-soft text-sm leading-relaxed">{description}</p>

          <div>
            <h4 className="text-xs font-mono font-bold text-text-muted tracking-wider mb-3 uppercase">
              Technical Highlights
            </h4>
            <ul className="space-y-2">
              {highlights.map((h, i) => (
                <li key={i} className="flex items-start gap-2 text-sm text-text-soft">
                  <span className="text-gold mt-0.5 shrink-0">▸</span>
                  {h}
                </li>
              ))}
            </ul>
          </div>

          <div className="flex flex-wrap gap-1.5">
            {tags.map((tag) => (
              <span key={tag} className="tag">{tag}</span>
            ))}
          </div>

          {githubUrl && (
            <a
              href={githubUrl}
              target="_blank"
              className="inline-flex items-center gap-2 text-xs font-mono font-bold text-blue hover:text-blue/80 transition-colors"
            >
              View source →
            </a>
          )}
        </div>

        <div className="flex-1 min-w-0">
          {demo}
        </div>
      </div>
    </section>
  );
}
