import { IDL } from '@dfinity/candid';

const t = IDL.Record({
  '\"' : IDL.Nat,
  '\'' : IDL.Nat,
  '\"\'' : IDL.Nat,
  '\\\n\'\"' : IDL.Nat,
});

export { t };


/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const idlFactory = ({ IDL }) => {
  const t = IDL.Record({
    '\"' : IDL.Nat,
    '\'' : IDL.Nat,
    '\"\'' : IDL.Nat,
    '\\\n\'\"' : IDL.Nat,
  });
  return IDL.Service({ '\n\'\"\'\'\"\"\r\t' : IDL.Func([t], [], []) });
};
/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => { return []; };
