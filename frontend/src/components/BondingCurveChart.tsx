/**
 * Bonding Curve Chart - Lightweight SVG visualization
 */

interface BondingCurveChartProps {
  progress: number;
  className?: string;
}

export function BondingCurveChart({ progress, className = '' }: BondingCurveChartProps) {
  // Generate quadratic curve points: y = x^2
  const points: string[] = [];
  for (let i = 0; i <= 100; i += 2) {
    points.push(`${i},${100 - (Math.pow(i/100, 2) * 100)}`);
  }
  const polylinePoints = points.join(' ');

  // Current position on curve
  const currentX = Math.min(progress, 100);
  const currentY = 100 - (Math.pow(currentX/100, 2) * 100);

  return (
    <div className={`w-full h-48 bg-white/5 rounded-xl p-4 relative overflow-hidden border border-white/10 ${className}`}>
      <div className="absolute top-4 left-4 text-xs font-mono text-purple-400 font-bold">
        BONDING CURVE PROGRESS
      </div>

      <svg className="w-full h-full" viewBox="0 0 100 100" preserveAspectRatio="none">
        {/* Gradient Definitions */}
        <defs>
          <linearGradient id="curveGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#d946ef" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#d946ef" stopOpacity="0" />
          </linearGradient>
        </defs>

        {/* Fill Area Under Curve */}
        <polygon
          points={`0,100 ${polylinePoints} 100,100`}
          fill="url(#curveGradient)"
        />

        {/* Curve Line */}
        <polyline
          points={polylinePoints}
          fill="none"
          stroke="#d946ef"
          strokeWidth="2"
          vectorEffect="non-scaling-stroke"
        />

        {/* Current Position Indicator */}
        <circle
          cx={currentX}
          cy={currentY}
          r="2"
          fill="white"
          className="animate-pulse"
          style={{ filter: 'drop-shadow(0 0 10px rgba(255,255,255,0.8))' }}
        />

        {/* Target Line (Graduation) */}
        <line
          x1="0"
          y1="10"
          x2="100"
          y2="10"
          stroke="rgba(255,255,255,0.2)"
          strokeDasharray="4"
          strokeWidth="1"
        />
      </svg>

      <div className="absolute top-2 right-4 text-xs text-green-400 font-bold">
        Target: Graduation 🎓
      </div>
    </div>
  );
}
