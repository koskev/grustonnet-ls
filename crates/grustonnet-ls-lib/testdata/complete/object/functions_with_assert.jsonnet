{
  local outerSelf = self,
  deploy(version, name, namespace):: {
    assert std.isNumber(version),
    assert std.isString(name),
    assert std.isString(namespace),
    version: version,
    metadata: {
      name: name,
      namespace: namespace,
    },

  },

  deployOuterAssert(version, name, namespace)::
    assert std.isNumber(version);
    assert std.isString(name);
    assert std.isString(namespace);
    {
      version: version,
      metadata: {
        name: name,
        namespace: namespace,
      },

    },

  x: outerSelf.deploy(1, '2', '3'),
  y: outerSelf.deployOuterAssert(1, '2', '3'),
}
