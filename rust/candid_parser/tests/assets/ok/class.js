import { IDL } from '@dfinity/candid';

const List = IDL.Rec();
List.fill(IDL.Opt(IDL.Tuple(IDL.Int, List)));
const Profile = IDL.Record({ 'age' : IDL.Nat8, 'name' : IDL.Text });

export { List };
export { Profile };


/**
 * @deprecated Use the individual type exports instead of the factory function.
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
 * @deprecated Use the individual type exports instead of the factory function.
 */
export const init = ({ IDL }) => {
  const List = IDL.Rec();
  List.fill(IDL.Opt(IDL.Tuple(IDL.Int, List)));
  const Profile = IDL.Record({ 'age' : IDL.Nat8, 'name' : IDL.Text });
  return [IDL.Int, List, Profile];
};
