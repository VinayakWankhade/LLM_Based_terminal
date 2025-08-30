import React from 'react';
import Terminal from './Terminal';

export type SplitDirection = 'vertical' | 'horizontal';

export type PaneNode =
  | {
      id: string;
      type: 'leaf';
      terminalId: string;
    }
  | {
      id: string;
      type: 'split';
      direction: SplitDirection; // vertical = left/right, horizontal = top/bottom
      children: PaneNode[]; // always length 2 for now
      sizes?: number[]; // optional flex ratios for children
    };

export interface PaneLayoutProps {
  node: PaneNode;
  activePaneId: string | null;
  onFocus: (paneId: string) => void;
  visible?: boolean; // whether the whole tab is visible
}

const PaneLayout: React.FC<PaneLayoutProps> = ({ node, activePaneId, onFocus, visible }) => {
  if (node.type === 'split') {
    const isRow = node.direction === 'horizontal';
    const sizes = node.sizes && node.sizes.length === 2 ? node.sizes : [1, 1];
    return (
      <div
        className={`pane-split pane-split-${node.direction}`}
        style={{
          display: 'flex',
          flexDirection: isRow ? 'column' : 'row',
        }}
      >
        <div className="pane-split-child" style={{ flex: sizes[0] }}>
          <PaneLayout node={node.children[0]} activePaneId={activePaneId} onFocus={onFocus} visible={visible} />
        </div>
        <div className={`pane-divider pane-divider-${node.direction}`} />
        <div className="pane-split-child" style={{ flex: sizes[1] }}>
          <PaneLayout node={node.children[1]} activePaneId={activePaneId} onFocus={onFocus} visible={visible} />
        </div>
      </div>
    );
  }

  // Leaf pane renders a Terminal
  const isActive = node.id === activePaneId;
  return (
    <div
      className={`pane-leaf ${isActive ? 'active' : ''}`}
      onClick={() => onFocus(node.id)}
    >
      <Terminal terminalId={node.terminalId} isVisible={!!visible} />
    </div>
  );
};

export default PaneLayout;
