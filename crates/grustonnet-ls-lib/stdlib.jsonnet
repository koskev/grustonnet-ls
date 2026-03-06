local html = import 'html.libsonnet';
local stdlib = import 'stdlib-content.jsonnet';

local addTypeAlias(name) = {
  assert std.isString(name),
  name: 'is%s' % std.asciiUpper(name[0:1]) + name[1:],
  params: ['x'],
  availableSince: '0.10.0',
  description: "Alias for std.type(x) == '%s'" % name,
};

local addFunction(name, params=[], description='', availableSince='0.10.0') = {
  name: name,
  params: params,
  availableSince: availableSince,
  description: description,
};
local version_0_21 = '0.21.0';

local extra_descriptions = {
  contains: 'This function is basically the same as <code>std.member</code> and can often be used interchangeably',
};

local modified_lib =
  {
    groups: [
      group {

        fields: [
          field {
            description: html.render(field.description) + std.get(extra_descriptions, field.name, ''),
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
          []
          + if group.id == 'math' then [

            addFunction('abs', ['n']),
            addFunction('sign', ['n']),
            addFunction('max', ['a', 'b']),
            addFunction('min', ['a', 'b']),
            addFunction('pow', ['x', 'n']),
            addFunction('exp', ['x'], description='Returns e^x'),
            addFunction('log', ['x']),
            addFunction('log2', ['x'], availableSince=version_0_21),
            addFunction('log10', ['x'], availableSince=version_0_21),
            addFunction('exponent', ['x'], description='Returns the exponent of the given floating point number'),
            addFunction('mantissa', ['x'], description='Returns the mantissa of the given floating point number'),
            addFunction('floor', ['x']),
            addFunction('ceil', ['x']),
            addFunction('sqrt', ['x']),
            addFunction('sin', ['x']),
            addFunction('cos', ['x']),
            addFunction('tan', ['x']),
            addFunction('asin', ['x']),
            addFunction('acos', ['x']),
            addFunction('atan', ['x']),
            addFunction('atan2', ['y', 'x'], availableSince=version_0_21),
            addFunction('deg2rad', ['x'], availableSince=version_0_21),
            addFunction('rad2deg', ['x'], availableSince=version_0_21),
            addFunction('hypot', ['a', 'b'], availableSince=version_0_21),
            addFunction('round', ['x'], availableSince=version_0_21),
            addFunction('isEven', ['x'], availableSince=version_0_21, description='Uses the integral part of the floating number'),
            addFunction('isOdd', ['x'], availableSince=version_0_21, description='Uses the integral part of the floating number'),
            addFunction('isInteger', ['x'], availableSince=version_0_21),
            addFunction('isDecimal', ['x'], availableSince=version_0_21),
            addFunction('pi', availableSince=version_0_21, description='Note: This is a field not a function'),
            addFunction('mod', ['a', 'b'], description='Performs modulo arithmetic if the left hand side is a number, or if the left hand side is a string, it does Python-style string formatting with std.format()'),
          ]
          else [],
      }
      for group in stdlib.groups

    ],
  };

stdlib + modified_lib
