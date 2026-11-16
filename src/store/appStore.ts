import { create } from 'zustand';

export type PovSlot = {
  id: number;
  label: string;
  /** 已载入的 demo 路径；空串表示该路尚未绑定 */
  source: string;
};

const SLOT_COUNT = 5;

const initialSlots: PovSlot[] = Array.from({ length: SLOT_COUNT }, (_, index) => ({
  id: index + 1,
  label: `POV ${index + 1}`,
  source: '',
}));

type AppState = {
  slots: PovSlot[];
  activeSlot: number;
  bindSource: (id: number, source: string) => void;
  setActiveSlot: (id: number) => void;
  loadedCount: () => number;
};

export const useAppStore = create<AppState>((set, get) => ({
  slots: initialSlots,
  activeSlot: 1,
  bindSource: (id, source) =>
    set((state) => ({
      slots: state.slots.map((slot) => (slot.id === id ? { ...slot, source } : slot)),
    })),
  setActiveSlot: (id) => set({ activeSlot: id }),
  loadedCount: () => get().slots.filter((slot) => slot.source !== '').length,
}));
