local testConditional(condition) = if condition then { truePath: true } else { falsePath: false };
{
  x: testConditional(true),
}
