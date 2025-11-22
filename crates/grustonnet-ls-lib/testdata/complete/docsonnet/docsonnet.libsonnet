{
  local outerSelf = self,
  val(type, help, default):: {
    assert std.isString(help),
    assert std.contains(outerSelf.types, type),
    help: help,
    type: type,
    default: default,
  },

  argument(type, name, default):: {
    assert std.contains(outerSelf.types, type),
    assert std.isString(name),
    type: type,
    name: name,
    default: default,
  },

  fn(help, args):: {
    assert std.isString(help),
    assert std.isArray(args),
    'function': {
      help: help,
      args: args,
    },
  },

  object(help, fields):: {
    assert std.isString(help),
    assert std.isObject(fields),
    help: help,
    fields: fields,
  },

  field(func, object, value):: {
    'function': func,
    object: object,
    value: value,
  },

  package(name, imp, help, api, sub):: {
    assert std.isString(name),
    assert std.isString(imp),
    assert std.isString(help),
    assert std.isObject(api),
    name: name,
    'import': imp,
    help: help,

    api: api,
    sub: sub,
  },

  types: [
    'string',
    'number',
    'boolean',
    'object',
    'array',
    'any',
    'function',
  ],
}
