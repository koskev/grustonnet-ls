local html = import 'html.libsonnet';
local stdlib = import 'stdlib-content.jsonnet';

local modified_lib =
  {
    groups: [
      group {

        fields: [
          field {
            description: html.render(field.description),
          }
          for field in group.fields
        ],
      }
      for group in stdlib.groups

    ],
  };

stdlib + modified_lib
