{
  local maxsplits = 5,
  local delimiter = ' ',
  singleLine: std.splitLimitR('', ' ', 3),
  withVar: std.splitLimitR('', delimiter, maxsplits),
  multiLine: std.splitLimitR(
    '',
    ' ',
    3
  ),
}
