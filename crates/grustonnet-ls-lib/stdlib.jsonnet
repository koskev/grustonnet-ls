local html = import 'html.libsonnet';
local stdlib = import 'stdlib-content.jsonnet';

local addTypeAlias(name) = {
  assert std.isString(name),
  name: 'is%s' % std.asciiUpper(name[0:1]) + name[1:],
  params: ['x'],
  availableSince: '0.10.0',
  description: "Alias for std.type(x) == '%s'" % name,
};

local modified_lib =
  {
    groups: [
      group {

        fields: [
          field {
            description: html.render(field.description),
          }
          for field in group.fields
        ] + if group.id == 'types_reflection' then [
          // XXX: The documentation is incomplete for those aliases. Therefore we need to add them manually
          addTypeAlias('array'),
          addTypeAlias('boolean'),
          addTypeAlias('function'),
          addTypeAlias('number'),
          addTypeAlias('object'),
          addTypeAlias('string'),
        ] else
          [],
      }
      for group in stdlib.groups

    ],
  };

stdlib + modified_lib
