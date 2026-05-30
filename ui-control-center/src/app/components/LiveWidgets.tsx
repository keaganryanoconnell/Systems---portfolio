"use client";

import GitHubCommits from "./widgets/GitHubCommits";
import SpotifyNowPlaying from "./widgets/SpotifyNowPlaying";
import LocalDashboard from "./widgets/LocalDashboard";

export default function LiveWidgets() {
  return (
    <section className="section">
      <div className="section-heading">Live Data</div>
      <h2 className="section-title mb-8">Real-Time Dashboard</h2>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <GitHubCommits />
        <SpotifyNowPlaying />
        <LocalDashboard />
      </div>
    </section>
  );
}
