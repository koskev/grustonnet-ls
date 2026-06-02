local checkType(node, val) = if std.type(node) == val then { trueKey: 1 } else { falseKey: 1 };
local checkVal(bool) = if bool then { trueKey: 1 } else { falseKey: 1 };

{
  x: checkType([], 'string'),
  y: checkVal(std.isObject('')),
}
