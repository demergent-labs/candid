import { IDL } from '@dfinity/candid';

const List = IDL.Rec();
List.fill(IDL.Opt(IDL.Tuple(IDL.Int, List)));
export { List };
const Profile = IDL.Record({ 'age' : IDL.Nat8, 'name' : IDL.Text });
export { Profile };

export const idlService = IDL.Service({
  'get' : IDL.Func([], [List], []),
  'set' : IDL.Func([List], [List], []),
});

export const idlInit = [IDL.Int, List, Profile];

/**
 * @deprecated Import IDL types directly from this module instead of using this factory function.
 */
export const idlFactory = ({ IDL }) => {
  const List = IDL.Rec();
  List.fill(IDL.Opt(IDL.Tuple(IDL.Int, List)));
  const Profile = IDL.Record({ 'age' : IDL.Nat8, 'name' : IDL.Text });
  return IDL.Service({
    'get' : IDL.Func([], [List], []),
    'set' : IDL.Func([List], [List], []),
  });
};
/**
 * @deprecated Import IDL types directly from this module instead of using this factory function.
 */
export const init = ({ IDL }) => {
  const List = IDL.Rec();
  List.fill(IDL.Opt(IDL.Tuple(IDL.Int, List)));
  const Profile = IDL.Record({ 'age' : IDL.Nat8, 'name' : IDL.Text });
  return [IDL.Int, List, Profile];
};
