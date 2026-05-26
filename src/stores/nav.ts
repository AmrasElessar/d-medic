import { defineStore } from 'pinia';

export type ViewKey =
  | 'dashboard'
  | 'scan'
  | 'plan'
  | 'execute'
  | 'guides'
  | 'history'
  | 'uninstall'
  | 'defrag'
  | 'settings'
  | 'about';

interface NavState {
  current: ViewKey;
  history: ViewKey[];
}

export const useNavStore = defineStore('nav', {
  state: (): NavState => ({
    current: 'dashboard',
    history: ['dashboard'],
  }),
  actions: {
    go(view: ViewKey): void {
      if (this.current === view) return;
      this.history.push(view);
      this.current = view;
    },
    back(): void {
      if (this.history.length <= 1) return;
      this.history.pop();
      this.current = this.history[this.history.length - 1]!;
    },
  },
});
