import { Store } from '../model/store';

export function renderList(store: Store): string[] {
  return store.titles();
}
