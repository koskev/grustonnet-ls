local microHtmlToMarkdown(html) =
  std.foldl(function(str, rep) std.strReplace(str, '<%s>' % rep.key, rep.value), std.objectKeysValues({
    code: '`',
    '/code': '`',
    tt: '`',
    '/tt': '`',
    em: '*',
    '/em': '*',
    p: '',
    '/p': '\n',
    ul: '- ',
    '/ul': '',
    pre: '',
    '/pre': '',
  }), html);

{
  render: function(x) microHtmlToMarkdown(if std.isString(x) then x else std.join('\n', x)),
  paragraphs: function(list) std.join('\n', list),
  p: function(_attrs, html) html + '\n',
  pre: function(_attrs, html) '`%s`' % html,
  spaceless: function(list) std.join('', list),
  // TODO std.escapeStringXml
  escape: function(str) std.strReplace(
    std.strReplace(
      std.strReplace(str, '&', '&amp;'),
      '<',
      '&lt;'
    ),
    '>',
    '&gt;'
  ),
}
