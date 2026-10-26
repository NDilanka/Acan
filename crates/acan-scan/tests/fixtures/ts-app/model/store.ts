import { Todo } from './todo';
import { format } from '../util/format';

export class Store {
  private items: Todo[] = [];

  add(todo: Todo): void {
    this.items.push(todo);
  }

  titles(): string[] {
    return this.items.map((todo) => format(todo.title));
  }
}
