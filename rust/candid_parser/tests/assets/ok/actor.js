import { IDL } from '@dfinity/candid';

const o = IDL.Rec();
const f = IDL.Func([IDL.Int8], [IDL.Int8], []);
const h = IDL.Func([f], [f], []);
const g = f;
o.fill(IDL.Opt(o));

export { f };
export { h };
export { g };
export { o };


/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const idlFactory = ({ IDL }) => {
  const o = IDL.Rec();
  const f = IDL.Func([IDL.Int8], [IDL.Int8], []);
  const h = IDL.Func([f], [f], []);
  const g = f;
  o.fill(IDL.Opt(o));
  return IDL.Service({
    'f' : IDL.Func([IDL.Nat], [h], []),
    'g' : f,
    'h' : g,
    'h2' : h,
    'o' : IDL.Func([o], [o], []),
  });
};
/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => { return []; };
