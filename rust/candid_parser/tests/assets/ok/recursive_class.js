import { IDL } from '@dfinity/candid';

const s = IDL.Rec();
s.fill(IDL.Service({ 'next' : IDL.Func([], [s], []) }));

export { s };


/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const idlFactory = ({ IDL }) => {
  const s = IDL.Rec();
  s.fill(IDL.Service({ 'next' : IDL.Func([], [s], []) }));
  return s.getType();
};
/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => {
  const s = IDL.Rec();
  s.fill(IDL.Service({ 'next' : IDL.Func([], [s], []) }));
  return [s];
};
