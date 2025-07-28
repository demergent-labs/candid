import { IDL } from '@dfinity/candid';

const A = IDL.Rec();
const C = A;
const B = IDL.Opt(C);
A.fill(IDL.Opt(B));
const Z = A;
const Y = Z;
const X = Y;

export { C };
export { B };
export { A };
export { Z };
export { Y };
export { X };


/**
 * @deprecated Use the individual type exports instead of the factory function.
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
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => { return []; };
