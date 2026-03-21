import React, { useState, useRef, useEffect, useCallback } from 'react';
import type { ChainConfig } from '../dsp/types';
import type { Preset } from '../dsp/presets';
import { getAllPresets, savePreset, deletePreset } from '../dsp/presets';

interface PresetBarProps {
  config: ChainConfig;
  onLoadPreset: (config: ChainConfig, name: string) => void;
}

export function PresetBar({ config, onLoadPreset }: PresetBarProps) {
  const [presets, setPresets] = useState<Preset[]>(() => getAllPresets());
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isSaving, setIsSaving] = useState(false);
  const [saveName, setSaveName] = useState('');
  const [isDropdownOpen, setIsDropdownOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const refresh = useCallback(() => {
    setPresets(getAllPresets());
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!isDropdownOpen) return;
    const handler = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setIsDropdownOpen(false);
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, [isDropdownOpen]);

  // Focus input when entering save mode
  useEffect(() => {
    if (isSaving && inputRef.current) inputRef.current.focus();
  }, [isSaving]);

  const current = presets[selectedIndex] || presets[0];

  const navigate = (delta: number) => {
    const next = (selectedIndex + delta + presets.length) % presets.length;
    setSelectedIndex(next);
    const preset = presets[next];
    if (preset) onLoadPreset(preset.config, preset.name);
  };

  const handleSelect = (index: number) => {
    setSelectedIndex(index);
    setIsDropdownOpen(false);
    const preset = presets[index];
    if (preset) onLoadPreset(preset.config, preset.name);
  };

  const handleSave = () => {
    if (!isSaving) {
      setIsSaving(true);
      setSaveName('');
      return;
    }
    const name = saveName.trim().toUpperCase();
    if (!name) {
      setIsSaving(false);
      return;
    }
    savePreset(name, config);
    refresh();
    // Select the newly saved preset
    const updated = getAllPresets();
    const idx = updated.findIndex(p => p.name === name);
    if (idx >= 0) setSelectedIndex(idx);
    setIsSaving(false);
  };

  const handleSaveKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') handleSave();
    if (e.key === 'Escape') setIsSaving(false);
  };

  const handleDelete = () => {
    if (!current || current.factory) return;
    deletePreset(current.name);
    refresh();
    setSelectedIndex(Math.max(0, selectedIndex - 1));
    const updated = getAllPresets();
    const prev = updated[Math.max(0, selectedIndex - 1)];
    if (prev) onLoadPreset(prev.config, prev.name);
  };

  // Count factory vs user
  const factoryCount = presets.filter(p => p.factory).length;

  return (
    <div className="flex items-stretch gap-0 rounded-lg border border-gray-800 bg-gray-900/80 overflow-hidden h-9">
      {/* Prev button */}
      <button
        onClick={() => navigate(-1)}
        className="px-2.5 text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors text-sm border-r border-gray-800"
        title="Previous preset"
      >
        &#9664;
      </button>

      {/* Display / dropdown */}
      <div className="relative flex-1 min-w-0" ref={dropdownRef}>
        <button
          onClick={() => setIsDropdownOpen(!isDropdownOpen)}
          className="w-full h-full flex items-center px-3 gap-2 hover:bg-gray-800/50 transition-colors"
        >
          {/* Preset number */}
          <span className="text-[10px] text-gray-600 tabular-nums w-6 text-right shrink-0">
            {String(selectedIndex + 1).padStart(2, '0')}
          </span>

          {/* LCD-style name display */}
          <span className="text-xs tracking-wider text-teal-400 truncate font-medium">
            {current?.name || '---'}
          </span>

          {/* Factory/user badge */}
          {current && (
            <span className={`text-[9px] px-1 py-px rounded shrink-0 ${
              current.factory
                ? 'bg-gray-700 text-gray-500'
                : 'bg-teal-500/15 text-teal-500'
            }`}>
              {current.factory ? 'FCT' : 'USR'}
            </span>
          )}

          <span className="flex-1" />
          <span className="text-[10px] text-gray-600">
            {isDropdownOpen ? '\u25B2' : '\u25BC'}
          </span>
        </button>

        {/* Dropdown list */}
        {isDropdownOpen && (
          <div className="absolute top-full left-0 right-0 z-50 mt-0.5 rounded-lg border border-gray-700 bg-gray-900 shadow-xl max-h-72 overflow-y-auto">
            {/* Factory presets */}
            <div className="px-2 pt-1.5 pb-0.5">
              <span className="text-[9px] uppercase tracking-widest text-gray-600">Factory</span>
            </div>
            {presets.slice(0, factoryCount).map((p, i) => (
              <button
                key={`f-${i}`}
                onClick={() => handleSelect(i)}
                className={`w-full text-left px-3 py-1.5 flex items-center gap-2 text-xs transition-colors ${
                  i === selectedIndex
                    ? 'bg-teal-500/10 text-teal-400'
                    : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'
                }`}
              >
                <span className="text-[10px] text-gray-600 tabular-nums w-5 text-right">
                  {String(i + 1).padStart(2, '0')}
                </span>
                <span className="truncate">{p.name}</span>
              </button>
            ))}

            {/* User presets */}
            {presets.length > factoryCount && (
              <>
                <div className="px-2 pt-2 pb-0.5 border-t border-gray-800">
                  <span className="text-[9px] uppercase tracking-widest text-gray-600">User</span>
                </div>
                {presets.slice(factoryCount).map((p, i) => {
                  const globalIdx = factoryCount + i;
                  return (
                    <button
                      key={`u-${i}`}
                      onClick={() => handleSelect(globalIdx)}
                      className={`w-full text-left px-3 py-1.5 flex items-center gap-2 text-xs transition-colors ${
                        globalIdx === selectedIndex
                          ? 'bg-teal-500/10 text-teal-400'
                          : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'
                      }`}
                    >
                      <span className="text-[10px] text-gray-600 tabular-nums w-5 text-right">
                        {String(globalIdx + 1).padStart(2, '0')}
                      </span>
                      <span className="truncate">{p.name}</span>
                    </button>
                  );
                })}
              </>
            )}
          </div>
        )}
      </div>

      {/* Next button */}
      <button
        onClick={() => navigate(1)}
        className="px-2.5 text-gray-500 hover:text-gray-300 hover:bg-gray-800 transition-colors text-sm border-l border-r border-gray-800"
        title="Next preset"
      >
        &#9654;
      </button>

      {/* Save button / input */}
      {isSaving ? (
        <div className="flex items-center border-l border-gray-800">
          <input
            ref={inputRef}
            value={saveName}
            onChange={e => setSaveName(e.target.value)}
            onKeyDown={handleSaveKeyDown}
            onBlur={() => { if (!saveName.trim()) setIsSaving(false); }}
            placeholder="NAME..."
            maxLength={20}
            className="w-28 bg-gray-800 text-teal-400 text-xs px-2 h-full outline-none placeholder-gray-600 tracking-wider"
          />
          <button
            onClick={handleSave}
            className="px-2.5 text-teal-500 hover:text-teal-300 hover:bg-gray-800 transition-colors text-[10px] font-bold h-full"
          >
            OK
          </button>
          <button
            onClick={() => setIsSaving(false)}
            className="px-2 text-gray-600 hover:text-gray-400 hover:bg-gray-800 transition-colors text-[10px] h-full"
          >
            ESC
          </button>
        </div>
      ) : (
        <button
          onClick={handleSave}
          className="px-3 text-gray-500 hover:text-teal-400 hover:bg-gray-800 transition-colors text-[10px] font-bold tracking-wider border-l border-gray-800"
          title="Save current settings as preset"
        >
          SAVE
        </button>
      )}

      {/* Delete button (user presets only) */}
      {current && !current.factory && !isSaving && (
        <button
          onClick={handleDelete}
          className="px-2.5 text-gray-600 hover:text-red-400 hover:bg-gray-800 transition-colors text-[10px] border-l border-gray-800"
          title="Delete preset"
        >
          DEL
        </button>
      )}
    </div>
  );
}
