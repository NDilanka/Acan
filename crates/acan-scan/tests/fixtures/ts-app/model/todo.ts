export interface Todo {
  id: number;
  title: string;
  done: boolean;
}

export type TodoId = number;

export const makeTodo = (title: string): Todo => ({ id: 0, title, done: false });

class Clock {
  tick(): number {
    return nextId++;
  }
}

class Hidden {
  secret(): void {}
}

const nextId = 1;

export { Clock as Timer };
