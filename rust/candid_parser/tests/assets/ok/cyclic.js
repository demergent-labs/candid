import { IDL } from '@dfinity/candid';

const A = IDL.Rec();
const C = A;
export { C };
const B = IDL.Opt(C);
export { B };
A.fill(IDL.Opt(B));
export { A };
const Z = A;
export { Z };
const Y = Z;
export { Y };
const X = Y;
export { X };

export const idlService = IDL.Service({
  'f' : IDL.Func([A, B, C, X, Y, Z], [], []),
});

export const idlInit = [];

/**
 * @deprecated Import IDL types directly from this module instead of using this factory function.
 */
export const idlFactory = ({ IDL }) => {
  const A = IDL.Rec();
  const C = A;
  const B = IDL.Opt(C);
  A.fill(IDL.Opt(B));
  const Z = A;
  const Y = Z;
  const X = Y;
  return IDL.Service({ 'f' : IDL.Func([A, B, C, X, Y, Z], [], []) });
};
/**
 * @deprecated Import IDL types directly from this module instead of using this factory function.
 */
export const init = ({ IDL }) => { return []; };
