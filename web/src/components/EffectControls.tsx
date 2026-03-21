import React from 'react';
import { Toggle, Slider, PillToggle, Select, EffectSection } from './Controls';
import type { ChainConfig } from '../dsp/types';
import { EFFECT_REGISTRY, type EffectDef } from '../dsp/effectRegistry';

interface EffectControlsProps {
  config: ChainConfig;
  onChange: (config: ChainConfig) => void;
  hasReference: boolean;
}

function EffectControl({ effect, config, onChange }: {
  effect: EffectDef;
  config: ChainConfig;
  onChange: (config: ChainConfig) => void;
}) {
  const id = effect.id as keyof ChainConfig;
  const active = config[id] != null;
  const effectConfig = config[id] as Record<string, unknown> | null;

  const handleToggle = (on: boolean) => {
    if (on) {
      const defaults = { ...effect.defaults };
      if (effect.hasSeed) defaults.seed = Math.floor(Math.random() * 10000);
      onChange({ ...config, [id]: defaults });
    } else {
      onChange({ ...config, [id]: null });
    }
  };

  const updateParam = (key: string, value: unknown) => {
    onChange({ ...config, [id]: { ...effectConfig!, [key]: value } });
  };

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2 flex-wrap">
        <Toggle label={effect.label} active={active} onChange={handleToggle} />
        {active && effect.toggles?.map(t => (
          <PillToggle
            key={t.key}
            label={t.label}
            active={effectConfig?.[t.key] as boolean ?? t.defaultValue}
            onChange={(v) => updateParam(t.key, v)}
          />
        ))}
        {active && effect.hasSeed && (
          <PillToggle label="REROLL" active={false} onChange={() => updateParam('seed', Math.floor(Math.random() * 10000))} />
        )}
      </div>
      {active && effect.params.map(p => {
        if (p.type === 'slider') {
          return (
            <Slider
              key={p.key}
              label={p.label}
              value={effectConfig?.[p.key] as number ?? p.defaultValue as number}
              min={p.min!}
              max={p.max!}
              step={p.step}
              unit={p.unit}
              onChange={(v) => updateParam(p.key, v)}
            />
          );
        }
        if (p.type === 'select') {
          return (
            <Select
              key={p.key}
              label={p.label}
              value={effectConfig?.[p.key] as string ?? p.defaultValue as string}
              options={p.options!}
              onChange={(v) => updateParam(p.key, v)}
            />
          );
        }
        return null;
      })}
      {active && effect.description && (
        <p className={`text-[10px] pl-[16ch] ${
          effect.id === 'fpDisrupt' ? 'text-rose-400/70' : 'text-gray-500'
        }`}>
          {effect.description}
        </p>
      )}
    </div>
  );
}

export function EffectControls({ config, onChange, hasReference }: EffectControlsProps) {
  const modifying = EFFECT_REGISTRY.filter(e => e.category === 'modifying');
  const mastering = EFFECT_REGISTRY.filter(e => e.category === 'mastering');

  const modCount = modifying.filter(e => config[e.id as keyof ChainConfig] != null).length;
  const masterCount = mastering.filter(e => config[e.id as keyof ChainConfig] != null).length;

  // Filter out refWarp if no reference loaded
  const visibleModifying = modifying.filter(e => e.id !== 'refWarp' || hasReference);

  return (
    <div className="space-y-2">
      <EffectSection title="MODIFYING" activeCount={modCount} tagColor="bg-blue-500/20 text-blue-400">
        {visibleModifying.map(effect => (
          <EffectControl key={effect.id} effect={effect} config={config} onChange={onChange} />
        ))}
      </EffectSection>

      <EffectSection title="MASTERING" activeCount={masterCount} tagColor="bg-amber-500/20 text-amber-400">
        {mastering.map((effect, i) => (
          <div key={effect.id} className={effect.id === 'fpDisrupt' ? 'border-t border-gray-800 pt-3' : ''}>
            <EffectControl effect={effect} config={config} onChange={onChange} />
          </div>
        ))}
      </EffectSection>
    </div>
  );
}
