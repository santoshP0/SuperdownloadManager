import { useEffect, useRef, useState } from "react";

const MAX_POINTS = 60; // 30 seconds at 500 ms per tick

export function useSpeedHistory(currentSpeed: number): number[] {
  const [history, setHistory] = useState<number[]>(() =>
    Array(MAX_POINTS).fill(0)
  );
  const ref = useRef(currentSpeed);
  ref.current = currentSpeed;

  useEffect(() => {
    const id = setInterval(() => {
      setHistory((prev) => [...prev.slice(1), ref.current]);
    }, 500);
    return () => clearInterval(id);
  }, []);

  return history;
}
