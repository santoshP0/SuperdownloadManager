interface Props {
  percent: number;
  status: string;
}

const colorMap: Record<string, string> = {
  downloading: "bg-blue-500",
  retrying: "bg-orange-500",
  completed: "bg-green-500",
  paused: "bg-yellow-500",
  failed: "bg-red-500",
  queued: "bg-slate-500",
  cancelled: "bg-slate-600",
};

export function ProgressBar({ percent, status }: Props) {
  const color = colorMap[status] ?? "bg-slate-500";
  return (
    <div className="w-full h-1.5 bg-slate-700 rounded-full overflow-hidden">
      <div
        className={`h-full rounded-full transition-all duration-300 ${color}`}
        style={{ width: `${percent}%` }}
      />
    </div>
  );
}
