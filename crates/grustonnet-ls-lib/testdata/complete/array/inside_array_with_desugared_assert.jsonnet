local myVar = { key: 5 };

[
  if true then (
    myVar // var
  ) else error 'Assertion failed',
]
