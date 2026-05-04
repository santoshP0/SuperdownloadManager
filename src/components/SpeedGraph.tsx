interface Props {
  history: number[];
  width?: number;
  height?: number;
}

export function SpeedGraph({ history, width = 120, height = 34 }: Props) {
  const n = history.length;
  if (n < 2) return null;

  const peak = Math.max(...history, 1024); // floor at 1 KB/s so graph isn't flat on zero

  const pts = history.map((v, i): [number, number] => [
    (i / (n - 1)) * width,
    height - (v / peak) * (height - 2) - 1,
  ]);

  const line = pts
    .map(([x, y], i) => `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`)
    .join(" ");

  const area = `${line} L${width},${height} L0,${height} Z`;

  return (
    <svg
      width={width}
      height={height}
      className="overflow-visible shrink-0"
      aria-hidden
    >
      <defs>
        <linearGradient id="sgFill" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%"   stopColor="#3b82f6" stopOpacity="0.4" />
          <stop offset="100%" stopColor="#3b82f6" stopOpacity="0.03" />
        </linearGradient>
      </defs>
      {/* Filled area */}
      <path d={area} fill="url(#sgFill)" />
      {/* Line */}
      <path
        d={line}
        fill="none"
        stroke="#3b82f6"
        strokeWidth="1.5"
        strokeLinejoin="round"
        strokeLinecap="round"
      />
    </svg>
  );
}
