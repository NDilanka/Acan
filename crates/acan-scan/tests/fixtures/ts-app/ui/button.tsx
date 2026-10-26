import React from 'react';
import { format } from '../util';

export interface ButtonProps {
  label: string;
}

export class Button extends React.Component<ButtonProps> {
  render() {
    return <button>{format(this.props.label)}</button>;
  }
}
