import React from 'react';
import { Composition } from 'remotion';
import { LedgerlyPromo, TOTAL_FRAMES } from './Main';
import { Fonts } from './lib/ui';

export type PromoProps = { bgm: boolean };

const Wrapper: React.FC<PromoProps> = ({ bgm }) => (
  <>
    <Fonts />
    <LedgerlyPromo bgm={bgm} />
  </>
);

export const Root: React.FC = () => (
  <Composition
    id="LedgerlyPromo"
    component={Wrapper}
    durationInFrames={TOTAL_FRAMES}
    fps={30}
    width={1920}
    height={1080}
    defaultProps={{ bgm: true }}
  />
);
