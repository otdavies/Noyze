import React from 'react';

interface ToggleProps {
  label: string;
  active: boolean;
  onChange: (active: boolean) => void;
}

export function Toggle({ label, active, onChange }: ToggleProps) {
  return (
    <button
      onClick={() => onChange(!active)}
      className={`min-w-[80px] px-3 py-1.5 rounded-full text-xs font-medium flex items-center gap-2 transition-colors ${
        active ? 'bg-teal-500/20 text-teal-400 border border-teal-500/40' : 'bg-gray-800 text-gray-500 border border-gray-700'
      }`}
    >
      <span className={`w-2 h-2 rounded-full ${active ? 'bg-teal-400' : 'bg-gray-600'}`} />
      {label}
    </button>
  );
}

interface SliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
}

export function Slider({ label, value, min, max, step = 0.01, unit = '', onChange }: SliderProps) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-gray-500 text-xs text-right w-[16ch] shrink-0">{label}</span>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="flex-1 min-w-[100px]"
      />
      <span className="text-gray-400 text-xs tabular-nums text-right w-[8ch] shrink-0">
        {typeof value === 'number' ? value.toFixed(step >= 1 ? 0 : 2) : value}{unit}
      </span>
    </div>
  );
}

interface PillToggleProps {
  label: string;
  active: boolean;
  onChange: (active: boolean) => void;
}

export function PillToggle({ label, active, onChange }: PillToggleProps) {
  return (
    <button
      onClick={() => onChange(!active)}
      className={`px-2 py-0.5 rounded text-[10px] font-medium transition-colors ${
        active ? 'bg-teal-500/20 text-teal-400' : 'bg-gray-800 text-gray-600'
      }`}
    >
      {label}
    </button>
  );
}

interface SelectProps {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

export function Select({ label, value, options, onChange }: SelectProps) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-gray-500 text-xs text-right w-[16ch] shrink-0">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="bg-gray-800 text-gray-200 text-xs border border-gray-700 rounded px-2 py-1"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>{o.label}</option>
        ))}
      </select>
    </div>
  );
}

interface EffectSectionProps {
  title: string;
  activeCount: number;
  tagColor: string;
  children: React.ReactNode;
  defaultOpen?: boolean;
}

export function EffectSection({ title, activeCount, tagColor, children, defaultOpen = false }: EffectSectionProps) {
  const [open, setOpen] = React.useState(defaultOpen);
  return (
    <div className="border border-gray-800 rounded-lg overflow-hidden">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between px-3 py-2 bg-gray-900/50 hover:bg-gray-900 transition-colors"
      >
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-400">{open ? '▼' : '▶'}</span>
          <span className="text-xs font-medium text-gray-300">{title}</span>
        </div>
        {activeCount > 0 && (
          <span className={`text-[10px] px-1.5 py-0.5 rounded ${tagColor}`}>
            {activeCount} active
          </span>
        )}
      </button>
      {open && <div className="p-3 space-y-3">{children}</div>}
    </div>
  );
}
