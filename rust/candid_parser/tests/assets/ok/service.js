import { IDL } from '@dfinity/candid';

const Service = IDL.Rec();
const Func = IDL.Func([], [Service], []);
Service.fill(IDL.Service({ 'f' : Func }));
const Service2 = Service;

export { Func };
export { Service };
export { Service2 };


/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const idlFactory = ({ IDL }) => {
  const Service = IDL.Rec();
  const Func = IDL.Func([], [Service], []);
  Service.fill(IDL.Service({ 'f' : Func }));
  const Service2 = Service;
  return IDL.Service({
    'asArray' : IDL.Func([], [IDL.Vec(Service2), IDL.Vec(Func)], ['query']),
    'asPrincipal' : IDL.Func([], [Service2, Func], []),
    'asRecord' : IDL.Func(
        [],
        [IDL.Tuple(Service2, IDL.Opt(Service), Func)],
        [],
      ),
    'asVariant' : IDL.Func(
        [],
        [
          IDL.Variant({
            'a' : Service2,
            'b' : IDL.Record({ 'f' : IDL.Opt(Func) }),
          }),
        ],
        [],
      ),
  });
};
/**
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => { return []; };
