export function rankSheetUrl(provider: string, tier: string): string {
  return `/sprites/${provider}-rank-${tier}.png`;
}

export function defaultSheet(provider: string): string {
  return provider === "claude"
    ? "/sprites/claude-fire-poison.png"
    : "/sprites/codex-ice-lightning.png";
}

/** Rank art lands asset-by-asset, so presence is probed instead of assumed. */
export function probeImage(url: string): Promise<boolean> {
  return new Promise((resolve) => {
    const image = new Image();
    image.onload = () => resolve(true);
    image.onerror = () => resolve(false);
    image.src = url;
  });
}

export async function resolveSheet(provider: string, tier: string): Promise<string> {
  const url = rankSheetUrl(provider, tier);
  return (await probeImage(url)) ? url : defaultSheet(provider);
}
