/*
Copyright 2015 Google Inc. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

/* This is the Jsonnet standard library, at least the parts of it that are written in Jsonnet.
 *
 * There are some native methods as well, which are defined in the interpreter and added to this
 * file.  It is never necessary to import stdlib.jsonnet, it is embedded into the interpreter at
 * compile-time and automatically imported into all other Jsonnet programs.
 */
{

  local stdlib = self,
  local id = function(x) x,

  local go_only_function = error 'This function is only supported in go version of jsonnet. See https://github.com/google/go-jsonnet',

  objectHasEx(o, f, x)::
    std.objectHasEx(o, f, x),

  isString(v):: stdlib.type(v) == 'string',
  isNumber(v):: stdlib.type(v) == 'number',
  isBoolean(v):: stdlib.type(v) == 'boolean',
  isObject(v):: stdlib.type(v) == 'object',
  isArray(v):: stdlib.type(v) == 'array',
  isFunction(v):: stdlib.type(v) == 'function',

  toString(a)::
    if stdlib.type(a) == 'string' then a else '' + a,

  substr(str, from, len)::
    assert stdlib.isString(str) : 'substr first parameter should be a string, got ' + stdlib.type(str);
    assert stdlib.isNumber(from) : 'substr second parameter should be a string, got ' + stdlib.type(from);
    assert stdlib.isNumber(len) : 'substr third parameter should be a string, got ' + stdlib.type(len);
    assert len >= 0 : 'substr third parameter should be greater than zero, got ' + len;
    stdlib.join('', stdlib.makeArray(stdlib.max(0, stdlib.min(len, stdlib.length(str) - from)), function(i) str[i + from])),

  startsWith(a, b)::
    if stdlib.length(a) < stdlib.length(b) then
      false
    else
      stdlib.substr(a, 0, stdlib.length(b)) == b,

  endsWith(a, b)::
    if stdlib.length(a) < stdlib.length(b) then
      false
    else
      stdlib.substr(a, stdlib.length(a) - stdlib.length(b), stdlib.length(b)) == b,

  lstripChars(str, chars)::
    if stdlib.length(str) > 0 && stdlib.member(chars, str[0]) then
      stdlib.lstripChars(str[1:], chars) tailstrict
    else
      str,

  rstripChars(str, chars)::
    local len = stdlib.length(str);
    if len > 0 && stdlib.member(chars, str[len - 1]) then
      stdlib.rstripChars(str[:len - 1], chars) tailstrict
    else
      str,

  stripChars(str, chars)::
    stdlib.lstripChars(stdlib.rstripChars(str, chars), chars),

  stringChars(str)::
    stdlib.makeArray(stdlib.length(str), function(i) str[i]),

  local parse_nat(str, base) =
    assert base > 0 && base <= 16 : 'integer base %d invalid' % base;
    // These codepoints are in ascending order:
    local zero_code = stdlib.codepoint('0');
    local upper_a_code = stdlib.codepoint('A');
    local lower_a_code = stdlib.codepoint('a');
    local addDigit(aggregate, char) =
      local code = stdlib.codepoint(char);
      local digit = if code >= lower_a_code then
        code - lower_a_code + 10
      else if code >= upper_a_code then
        code - upper_a_code + 10
      else
        code - zero_code;
      assert digit >= 0 && digit < base : '%s is not a base %d integer' % [str, base];
      base * aggregate + digit;
    stdlib.foldl(addDigit, stdlib.stringChars(str), 0),

  parseInt(str)::
    assert stdlib.isString(str) : 'Expected string, got ' + stdlib.type(str);
    assert stdlib.length(str) > 0 && str != '-' : 'Not an integer: "%s"' % [str];
    if str[0] == '-' then
      -parse_nat(str[1:], 10)
    else
      parse_nat(str, 10),

  parseOctal(str)::
    assert stdlib.isString(str) : 'Expected string, got ' + stdlib.type(str);
    assert stdlib.length(str) > 0 : 'Not an octal number: ""';
    parse_nat(str, 8),

  parseHex(str)::
    assert stdlib.isString(str) : 'Expected string, got ' + stdlib.type(str);
    assert stdlib.length(str) > 0 : 'Not hexadecimal: ""';
    parse_nat(str, 16),

  split(str, c)::
    assert stdlib.isString(str) : 'stdlib.split first parameter must be a String, got ' + stdlib.type(str);
    assert stdlib.isString(c) : 'stdlib.split second parameter must be a String, got ' + stdlib.type(c);
    assert stdlib.length(c) >= 1 : 'stdlib.split second parameter must have length 1 or greater, got ' + stdlib.length(c);
    stdlib.splitLimit(str, c, -1),

  splitLimit(str, c, maxsplits)::
    assert stdlib.isString(str) : 'str.splitLimit first parameter must be a String, got ' + stdlib.type(str);
    assert stdlib.isString(c) : 'str.splitLimit second parameter must be a String, got ' + stdlib.type(c);
    assert stdlib.length(c) >= 1 : 'stdlib.splitLimit second parameter must have length 1 or greater, got ' + stdlib.length(c);
    assert stdlib.isNumber(maxsplits) : 'str.splitLimit third parameter must be a Number, got ' + stdlib.type(maxsplits);
    local strLen = stdlib.length(str);
    local cLen = stdlib.length(c);
    local aux(idx, ret, val) =
      if idx >= strLen then
        ret + [val]
      else if str[idx:idx + cLen:1] == c &&
              (maxsplits == -1 || stdlib.length(ret) < maxsplits) then
        aux(idx + cLen, ret + [val], '')
      else
        aux(idx + 1, ret, val + str[idx]);
    aux(0, [], ''),

  splitLimitR(str, c, maxsplits)::
    assert stdlib.isString(str) : 'str.splitLimitR first parameter must be a String, got ' + stdlib.type(str);
    assert stdlib.isString(c) : 'str.splitLimitR second parameter must be a String, got ' + stdlib.type(c);
    assert stdlib.length(c) >= 1 : 'stdlib.splitLimitR second parameter must have length 1 or greater, got ' + stdlib.length(c);
    assert stdlib.isNumber(maxsplits) : 'str.splitLimitR third parameter must be a Number, got ' + stdlib.type(maxsplits);
    if maxsplits == -1 then
      stdlib.splitLimit(str, c, -1)
    else
      local revStr(str) = stdlib.join('', stdlib.reverse(stdlib.stringChars(str)));
      stdlib.map(function(e) revStr(e), stdlib.reverse(stdlib.splitLimit(revStr(str), revStr(c), maxsplits))),

  strReplace(str, from, to)::
    assert stdlib.isString(str);
    assert stdlib.isString(from);
    assert stdlib.isString(to);
    assert from != '' : "'from' string must not be zero length.";

    // Cache for performance.
    local str_len = stdlib.length(str);
    local from_len = stdlib.length(from);

    // True if from is at str[i].
    local found_at(i) = str[i:i + from_len] == from;

    // Return the remainder of 'str' starting with 'start_index' where
    // all occurrences of 'from' after 'curr_index' are replaced with 'to'.
    local replace_after(start_index, curr_index, acc) =
      if curr_index > str_len then
        acc + str[start_index:curr_index]
      else if found_at(curr_index) then
        local new_index = curr_index + stdlib.length(from);
        replace_after(new_index, new_index, acc + str[start_index:curr_index] + to) tailstrict
      else
        replace_after(start_index, curr_index + 1, acc) tailstrict;

    // if from_len==1, then we replace by splitting and rejoining the
    // string which is much faster than recursing on replace_after
    if from_len == 1 then
      stdlib.join(to, stdlib.split(str, from))
    else
      replace_after(0, 0, ''),

  asciiUpper(str)::
    local cp = stdlib.codepoint;
    local up_letter(c) = if cp(c) >= 97 && cp(c) < 123 then
      stdlib.char(cp(c) - 32)
    else
      c;
    stdlib.join('', stdlib.map(up_letter, stdlib.stringChars(str))),

  asciiLower(str)::
    local cp = stdlib.codepoint;
    local down_letter(c) = if cp(c) >= 65 && cp(c) < 91 then
      stdlib.char(cp(c) + 32)
    else
      c;
    stdlib.join('', stdlib.map(down_letter, stdlib.stringChars(str))),

  range(from, to)::
    stdlib.makeArray(to - from + 1, function(i) i + from),

  repeat(what, count)::
    local joiner =
      if stdlib.isString(what) then ''
      else if stdlib.isArray(what) then []
      else error 'stdlib.repeat first argument must be an array or a string';
    stdlib.join(joiner, stdlib.makeArray(count, function(i) what)),

  slice(indexable, index, end, step)::
    local invar =
      // loop invariant with defaults applied
      {
        indexable: indexable,
        index:
          if index == null
          then 0
          else
            if index < 0
            then stdlib.max(0, stdlib.length(indexable) + index)
            else index,
        end:
          if end == null
          then stdlib.length(indexable)
          else
            if end < 0
            then stdlib.length(indexable) + end
            else end,
        step:
          if step == null
          then 1
          else step,
        length: stdlib.length(indexable),
        type: stdlib.type(indexable),
      };
    assert invar.step >= 0 : 'got [%s:%s:%s] but negative steps are not supported' % [invar.index, invar.end, invar.step];
    assert step != 0 : 'got %s but step must be greater than 0' % step;
    assert stdlib.isString(indexable) || stdlib.isArray(indexable) : 'stdlib.slice accepts a string or an array, but got: %s' % stdlib.type(indexable);
    local build(slice, cur) =
      if cur >= invar.end || cur >= invar.length then
        slice
      else
        build(
          if invar.type == 'string' then
            slice + invar.indexable[cur]
          else
            slice + [invar.indexable[cur]],
          cur + invar.step
        ) tailstrict;
    build(if invar.type == 'string' then '' else [], invar.index),

  member(arr, x)::
    if stdlib.isArray(arr) then
      stdlib.count(arr, x) > 0
    else if stdlib.isString(arr) then
      stdlib.length(stdlib.findSubstr(x, arr)) > 0
    else error 'stdlib.member first argument must be an array or a string',

  count(arr, x):: stdlib.length(stdlib.filter(function(v) v == x, arr)),

  mod(a, b)::
    if stdlib.isNumber(a) && stdlib.isNumber(b) then
      stdlib.modulo(a, b)
    else if stdlib.isString(a) then
      stdlib.format(a, b)
    else
      error 'Operator % cannot be used on types ' + stdlib.type(a) + ' and ' + stdlib.type(b) + '.',

  // this is the most precision that will fit in a f64
  pi:: 3.14159265358979311600,

  deg2rad(x):: x * stdlib.pi / 180,
  rad2deg(x):: x * 180 / stdlib.pi,

  log2(x):: stdlib.log(x) / stdlib.log(2),
  log10(x):: stdlib.log(x) / stdlib.log(10),

  map(func, arr)::
    if !stdlib.isFunction(func) then
      error ('stdlib.map first param must be function, got ' + stdlib.type(func))
    else if !stdlib.isArray(arr) && !stdlib.isString(arr) then
      error ('stdlib.map second param must be array / string, got ' + stdlib.type(arr))
    else
      stdlib.makeArray(stdlib.length(arr), function(i) func(arr[i])),

  mapWithIndex(func, arr)::
    if !stdlib.isFunction(func) then
      error ('stdlib.mapWithIndex first param must be function, got ' + stdlib.type(func))
    else if !stdlib.isArray(arr) && !stdlib.isString(arr) then
      error ('stdlib.mapWithIndex second param must be array, got ' + stdlib.type(arr))
    else
      stdlib.makeArray(stdlib.length(arr), function(i) func(i, arr[i])),

  mapWithKey(func, obj)::
    if !stdlib.isFunction(func) then
      error ('stdlib.mapWithKey first param must be function, got ' + stdlib.type(func))
    else if !stdlib.isObject(obj) then
      error ('stdlib.mapWithKey second param must be object, got ' + stdlib.type(obj))
    else
      { [k]: func(k, obj[k]) for k in stdlib.objectFields(obj) },

  flatMap(func, arr)::
    if !stdlib.isFunction(func) then
      error ('stdlib.flatMap first param must be function, got ' + stdlib.type(func))
    else if stdlib.isArray(arr) then
      stdlib.flattenArrays(stdlib.makeArray(stdlib.length(arr), function(i) func(arr[i])))
    else if stdlib.isString(arr) then
      stdlib.join('', stdlib.makeArray(stdlib.length(arr), function(i) func(arr[i])))
    else error ('stdlib.flatMap second param must be array / string, got ' + stdlib.type(arr)),

  join(sep, arr)::
    local aux(arr, i, first, running) =
      if i >= stdlib.length(arr) then
        running
      else if arr[i] == null then
        aux(arr, i + 1, first, running) tailstrict
      else if stdlib.type(arr[i]) != stdlib.type(sep) then
        error 'expected %s but arr[%d] was %s ' % [stdlib.type(sep), i, stdlib.type(arr[i])]
      else if first then
        aux(arr, i + 1, false, running + arr[i]) tailstrict
      else
        aux(arr, i + 1, false, running + sep + arr[i]) tailstrict;
    if !stdlib.isArray(arr) then
      error 'join second parameter should be array, got ' + stdlib.type(arr)
    else if stdlib.isString(sep) then
      aux(arr, 0, true, '')
    else if stdlib.isArray(sep) then
      aux(arr, 0, true, [])
    else
      error 'join first parameter should be string or array, got ' + stdlib.type(sep),

  lines(arr)::
    stdlib.join('\n', arr + ['']),

  deepJoin(arr)::
    if stdlib.isString(arr) then
      arr
    else if stdlib.isArray(arr) then
      stdlib.join('', [stdlib.deepJoin(x) for x in arr])
    else
      error 'Expected string or array, got %s' % stdlib.type(arr),


  format(str, vals)::

    /////////////////////////////
    // Parse the mini-language //
    /////////////////////////////

    local try_parse_mapping_key(str, i) =
      assert i < stdlib.length(str) : 'Truncated format code.';
      local c = str[i];
      if c == '(' then
        local consume(str, j, v) =
          if j >= stdlib.length(str) then
            error 'Truncated format code.'
          else
            local c = str[j];
            if c != ')' then
              consume(str, j + 1, v + c)
            else
              { i: j + 1, v: v };
        consume(str, i + 1, '')
      else
        { i: i, v: null };

    local try_parse_cflags(str, i) =
      local consume(str, j, v) =
        assert j < stdlib.length(str) : 'Truncated format code.';
        local c = str[j];
        if c == '#' then
          consume(str, j + 1, v { alt: true })
        else if c == '0' then
          consume(str, j + 1, v { zero: true })
        else if c == '-' then
          consume(str, j + 1, v { left: true })
        else if c == ' ' then
          consume(str, j + 1, v { blank: true })
        else if c == '+' then
          consume(str, j + 1, v { plus: true })
        else
          { i: j, v: v };
      consume(str, i, { alt: false, zero: false, left: false, blank: false, plus: false });

    local try_parse_field_width(str, i) =
      if i < stdlib.length(str) && str[i] == '*' then
        { i: i + 1, v: '*' }
      else
        local consume(str, j, v) =
          assert j < stdlib.length(str) : 'Truncated format code.';
          local c = str[j];
          if c == '0' then
            consume(str, j + 1, v * 10 + 0)
          else if c == '1' then
            consume(str, j + 1, v * 10 + 1)
          else if c == '2' then
            consume(str, j + 1, v * 10 + 2)
          else if c == '3' then
            consume(str, j + 1, v * 10 + 3)
          else if c == '4' then
            consume(str, j + 1, v * 10 + 4)
          else if c == '5' then
            consume(str, j + 1, v * 10 + 5)
          else if c == '6' then
            consume(str, j + 1, v * 10 + 6)
          else if c == '7' then
            consume(str, j + 1, v * 10 + 7)
          else if c == '8' then
            consume(str, j + 1, v * 10 + 8)
          else if c == '9' then
            consume(str, j + 1, v * 10 + 9)
          else
            { i: j, v: v };
        consume(str, i, 0);

    local try_parse_precision(str, i) =
      assert i < stdlib.length(str) : 'Truncated format code.';
      local c = str[i];
      if c == '.' then
        try_parse_field_width(str, i + 1)
      else
        { i: i, v: null };

    // Ignored, if it exists.
    local try_parse_length_modifier(str, i) =
      assert i < stdlib.length(str) : 'Truncated format code.';
      local c = str[i];
      if c == 'h' || c == 'l' || c == 'L' then
        i + 1
      else
        i;

    local parse_conv_type(str, i) =
      assert i < stdlib.length(str) : 'Truncated format code.';
      local c = str[i];
      if c == 'd' || c == 'i' || c == 'u' then
        { i: i + 1, v: 'd', caps: false }
      else if c == 'o' then
        { i: i + 1, v: 'o', caps: false }
      else if c == 'x' then
        { i: i + 1, v: 'x', caps: false }
      else if c == 'X' then
        { i: i + 1, v: 'x', caps: true }
      else if c == 'e' then
        { i: i + 1, v: 'e', caps: false }
      else if c == 'E' then
        { i: i + 1, v: 'e', caps: true }
      else if c == 'f' then
        { i: i + 1, v: 'f', caps: false }
      else if c == 'F' then
        { i: i + 1, v: 'f', caps: true }
      else if c == 'g' then
        { i: i + 1, v: 'g', caps: false }
      else if c == 'G' then
        { i: i + 1, v: 'g', caps: true }
      else if c == 'c' then
        { i: i + 1, v: 'c', caps: false }
      else if c == 's' then
        { i: i + 1, v: 's', caps: false }
      else if c == '%' then
        { i: i + 1, v: '%', caps: false }
      else
        error 'Unrecognised conversion type: ' + c;


    // Parsed initial %, now the rest.
    local parse_code(str, i) =
      assert i < stdlib.length(str) : 'Truncated format code.';
      local mkey = try_parse_mapping_key(str, i);
      local cflags = try_parse_cflags(str, mkey.i);
      local fw = try_parse_field_width(str, cflags.i);
      local prec = try_parse_precision(str, fw.i);
      local len_mod = try_parse_length_modifier(str, prec.i);
      local ctype = parse_conv_type(str, len_mod);
      {
        i: ctype.i,
        code: {
          mkey: mkey.v,
          cflags: cflags.v,
          fw: fw.v,
          prec: prec.v,
          ctype: ctype.v,
          caps: ctype.caps,
        },
      };

    // Parse a format string (containing none or more % format tags).
    local parse_codes(str, i, out, cur) =
      if i >= stdlib.length(str) then
        out + [cur]
      else
        local c = str[i];
        if c == '%' then
          local r = parse_code(str, i + 1);
          parse_codes(str, r.i, out + [cur, r.code], '') tailstrict
        else
          parse_codes(str, i + 1, out, cur + c) tailstrict;

    local codes = parse_codes(str, 0, [], '');


    ///////////////////////
    // Format the values //
    ///////////////////////

    // Useful utilities
    local padding(w, s) =
      local aux(w, v) =
        if w <= 0 then
          v
        else
          aux(w - 1, v + s);
      aux(w, '');

    // Add s to the left of str so that its length is at least w.
    local pad_left(str, w, s) =
      padding(w - stdlib.length(str), s) + str;

    // Add s to the right of str so that its length is at least w.
    local pad_right(str, w, s) =
      str + padding(w - stdlib.length(str), s);

    // Render a sign & magnitude integer (radix ranges from decimal to binary).
    // neg should be a boolean, and when true indicates that we should render a negative number.
    // mag must always be a whole number >= 0, it's the magnitude of the integer to render
    // min_chars must be a whole number >= 0
    //   It is the field width, i.e. stdlib.length() of the result should be >= min_chars
    // min_digits must be a whole number >= 0. It's the number of zeroes to pad with.
    // blank must be a boolean, if true adds an additional ' ' in front of a positive number, so
    // that it is aligned with negative numbers with the same number of digits.
    // plus must be a boolean, if true adds a '+' in front of a positive number, so that it is
    // aligned with negative numbers with the same number of digits.  This takes precedence over
    // blank, if both are true.
    // radix must be a whole number >1 and <= 10.  It is the base of the system of numerals.
    // zero_prefix is a string prefixed before the sign to all numbers that are not 0.
    local render_int(neg, mag, min_chars, min_digits, blank, plus, radix, zero_prefix) =
      // dec is the minimal string needed to represent the number as text.
      local dec =
        if mag == 0 then
          '0'
        else
          local aux(n) =
            if n == 0 then
              zero_prefix
            else
              aux(stdlib.floor(n / radix)) + (n % radix);
          aux(mag);
      local zp = min_chars - (if neg || blank || plus then 1 else 0);
      local zp2 = stdlib.max(zp, min_digits);
      local dec2 = pad_left(dec, zp2, '0');
      (if neg then '-' else if plus then '+' else if blank then ' ' else '') + dec2;

    // Render an integer in hexadecimal.
    local render_hex(n__, min_chars, min_digits, blank, plus, add_zerox, capitals) =
      local numerals = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
                       + if capitals then ['A', 'B', 'C', 'D', 'E', 'F']
                       else ['a', 'b', 'c', 'd', 'e', 'f'];
      local n_ = stdlib.abs(n__);
      local aux(n) =
        if n == 0 then
          ''
        else
          aux(stdlib.floor(n / 16)) + numerals[n % 16];
      local hex = if stdlib.floor(n_) == 0 then '0' else aux(stdlib.floor(n_));
      local neg = n__ < 0;
      local zp = min_chars - (if neg || blank || plus then 1 else 0)
                 - (if add_zerox then 2 else 0);
      local zp2 = stdlib.max(zp, min_digits);
      local hex2 = (if add_zerox then (if capitals then '0X' else '0x') else '')
                   + pad_left(hex, zp2, '0');
      (if neg then '-' else if plus then '+' else if blank then ' ' else '') + hex2;

    local strip_trailing_zero(str) =
      local aux(str, i) =
        if i < 0 then
          ''
        else
          if str[i] == '0' then
            aux(str, i - 1)
          else
            stdlib.substr(str, 0, i + 1);
      aux(str, stdlib.length(str) - 1);

    // Render floating point in decimal form
    local render_float_dec(n__, zero_pad, blank, plus, ensure_pt, trailing, prec) =
      local n_ = stdlib.abs(n__);
      local whole = stdlib.floor(n_);
      // Represent the rounded number as an integer * 1/10**prec.
      // Note that it can also be equal to 10**prec and we'll need to carry
      // over to the wholes.  We operate on the absolute numbers, so that we
      // don't have trouble with the rounding direction.
      local denominator = stdlib.pow(10, prec);
      local numerator = stdlib.abs(n_) * denominator + 0.5;
      local whole = stdlib.sign(n_) * stdlib.floor(numerator / denominator);
      local frac = stdlib.floor(numerator) % denominator;
      local dot_size = if prec == 0 && !ensure_pt then 0 else 1;
      local zp = zero_pad - prec - dot_size;
      local str = render_int(n__ < 0, whole, zp, 0, blank, plus, 10, '');
      if prec == 0 then
        str + if ensure_pt then '.' else ''
      else
        if trailing || frac > 0 then
          local frac_str = render_int(false, frac, prec, 0, false, false, 10, '');
          str + '.' + if !trailing then strip_trailing_zero(frac_str) else frac_str
        else
          str;

    // Render floating point in scientific form
    local render_float_sci(n__, zero_pad, blank, plus, ensure_pt, trailing, caps, prec) =
      local exponent = if n__ == 0 then 0 else stdlib.floor(stdlib.log(stdlib.abs(n__)) / stdlib.log(10));
      local suff = (if caps then 'E' else 'e')
                   + render_int(exponent < 0, stdlib.abs(exponent), 3, 0, false, true, 10, '');
      local mantissa = if exponent == -324 then
        // Avoid a rounding error where stdlib.pow(10, -324) is 0
        // -324 is the smallest exponent possible.
        n__ * 10 / stdlib.pow(10, exponent + 1)
      else
        n__ / stdlib.pow(10, exponent);
      local zp2 = zero_pad - stdlib.length(suff);
      render_float_dec(mantissa, zp2, blank, plus, ensure_pt, trailing, prec) + suff;

    // Render a value with an arbitrary format code.
    local format_code(val, code, fw, prec_or_null, i) =
      local cflags = code.cflags;
      local fpprec = if prec_or_null != null then prec_or_null else 6;
      local iprec = if prec_or_null != null then prec_or_null else 0;
      local zp = if cflags.zero && !cflags.left then fw else 0;
      if code.ctype == 's' then
        stdlib.toString(val)
      else if code.ctype == 'd' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          render_int(val <= -1, stdlib.floor(stdlib.abs(val)), zp, iprec, cflags.blank, cflags.plus, 10, '')
      else if code.ctype == 'o' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          local zero_prefix = if cflags.alt then '0' else '';
          render_int(val <= -1, stdlib.floor(stdlib.abs(val)), zp, iprec, cflags.blank, cflags.plus, 8, zero_prefix)
      else if code.ctype == 'x' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          render_hex(stdlib.floor(val),
                     zp,
                     iprec,
                     cflags.blank,
                     cflags.plus,
                     cflags.alt,
                     code.caps)
      else if code.ctype == 'f' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          render_float_dec(val,
                           zp,
                           cflags.blank,
                           cflags.plus,
                           cflags.alt,
                           true,
                           fpprec)
      else if code.ctype == 'e' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          render_float_sci(val,
                           zp,
                           cflags.blank,
                           cflags.plus,
                           cflags.alt,
                           true,
                           code.caps,
                           fpprec)
      else if code.ctype == 'g' then
        if stdlib.type(val) != 'number' then
          error 'Format required number at '
                + i + ', got ' + stdlib.type(val)
        else
          local exponent = if val != 0 then stdlib.floor(stdlib.log(stdlib.abs(val)) / stdlib.log(10)) else 0;
          if exponent < -4 || exponent >= fpprec then
            render_float_sci(val,
                             zp,
                             cflags.blank,
                             cflags.plus,
                             cflags.alt,
                             cflags.alt,
                             code.caps,
                             fpprec - 1)
          else
            local digits_before_pt = stdlib.max(1, exponent + 1);
            render_float_dec(val,
                             zp,
                             cflags.blank,
                             cflags.plus,
                             cflags.alt,
                             cflags.alt,
                             fpprec - digits_before_pt)
      else if code.ctype == 'c' then
        if stdlib.type(val) == 'number' then
          stdlib.char(val)
        else if stdlib.type(val) == 'string' then
          if stdlib.length(val) == 1 then
            val
          else
            error '%c expected 1-sized string got: ' + stdlib.length(val)
        else
          error '%c expected number / string, got: ' + stdlib.type(val)
      else
        error 'Unknown code: ' + code.ctype;

    // Render a parsed format string with an array of values.
    local format_codes_arr(codes, arr, i, j, v) =
      if i >= stdlib.length(codes) then
        if j < stdlib.length(arr) then
          error ('Too many values to format: ' + stdlib.length(arr) + ', expected ' + j)
        else
          v
      else
        local code = codes[i];
        if stdlib.type(code) == 'string' then
          format_codes_arr(codes, arr, i + 1, j, v + code) tailstrict
        else
          local tmp = if code.fw == '*' then {
            j: j + 1,
            fw: if j >= stdlib.length(arr) then
              error ('Not enough values to format: ' + stdlib.length(arr) + ', expected at least ' + j)
            else
              arr[j],
          } else {
            j: j,
            fw: code.fw,
          };
          local tmp2 = if code.prec == '*' then {
            j: tmp.j + 1,
            prec: if tmp.j >= stdlib.length(arr) then
              error ('Not enough values to format: ' + stdlib.length(arr) + ', expected at least ' + tmp.j)
            else
              arr[tmp.j],
          } else {
            j: tmp.j,
            prec: code.prec,
          };
          local j2 = tmp2.j;
          local val =
            if j2 < stdlib.length(arr) then
              arr[j2]
            else
              error ('Not enough values to format: ' + stdlib.length(arr) + ', expected more than ' + j2);
          local s =
            if code.ctype == '%' then
              '%'
            else
              format_code(val, code, tmp.fw, tmp2.prec, j2);
          local s_padded =
            if code.cflags.left then
              pad_right(s, tmp.fw, ' ')
            else
              pad_left(s, tmp.fw, ' ');
          local j3 =
            if code.ctype == '%' then
              j2
            else
              j2 + 1;
          format_codes_arr(codes, arr, i + 1, j3, v + s_padded) tailstrict;

    // Render a parsed format string with an object of values.
    local format_codes_obj(codes, obj, i, v) =
      if i >= stdlib.length(codes) then
        v
      else
        local code = codes[i];
        if stdlib.type(code) == 'string' then
          format_codes_obj(codes, obj, i + 1, v + code) tailstrict
        else
          local f =
            if code.mkey == null then
              error 'Mapping keys required.'
            else
              code.mkey;
          local fw =
            if code.fw == '*' then
              error 'Cannot use * field width with object.'
            else
              code.fw;
          local prec =
            if code.prec == '*' then
              error 'Cannot use * precision with object.'
            else
              code.prec;
          local val =
            if stdlib.objectHasAll(obj, f) then
              obj[f]
            else
              error 'No such field: ' + f;
          local s =
            if code.ctype == '%' then
              '%'
            else
              format_code(val, code, fw, prec, f);
          local s_padded =
            if code.cflags.left then
              pad_right(s, fw, ' ')
            else
              pad_left(s, fw, ' ');
          format_codes_obj(codes, obj, i + 1, v + s_padded) tailstrict;

    if stdlib.isArray(vals) then
      format_codes_arr(codes, vals, 0, 0, '')
    else if stdlib.isObject(vals) then
      format_codes_obj(codes, vals, 0, '')
    else
      format_codes_arr(codes, [vals], 0, 0, ''),

  foldr(func, arr, init)::
    local aux(func, arr, running, idx) =
      if idx < 0 then
        running
      else
        aux(func, arr, func(arr[idx], running), idx - 1) tailstrict;
    aux(func, arr, init, stdlib.length(arr) - 1),

  foldl(func, arr, init)::
    local aux(func, arr, running, idx) =
      if idx >= stdlib.length(arr) then
        running
      else
        aux(func, arr, func(running, arr[idx]), idx + 1) tailstrict;
    aux(func, arr, init, 0),


  filterMap(filter_func, map_func, arr)::
    if !stdlib.isFunction(filter_func) then
      error ('stdlib.filterMap first param must be function, got ' + stdlib.type(filter_func))
    else if !stdlib.isFunction(map_func) then
      error ('stdlib.filterMap second param must be function, got ' + stdlib.type(map_func))
    else if !stdlib.isArray(arr) then
      error ('stdlib.filterMap third param must be array, got ' + stdlib.type(arr))
    else
      stdlib.map(map_func, stdlib.filter(filter_func, arr)),

  assertEqual(a, b)::
    // If the values are strings, escape them for printing.
    // If not, they'll be JSON-stringified anyway by the later string concatenation.
    local astr = if stdlib.type(a) == 'string' then stdlib.escapeStringJson(a) else a;
    local bstr = if stdlib.type(b) == 'string' then stdlib.escapeStringJson(b) else b;
    if a == b then
      true
    else
      error 'Assertion failed. ' + astr + ' != ' + bstr,

  abs(n)::
    if !stdlib.isNumber(n) then
      error 'stdlib.abs expected number, got ' + stdlib.type(n)
    else
      if n > 0 then n else -n,

  sign(n)::
    if !stdlib.isNumber(n) then
      error 'stdlib.sign expected number, got ' + stdlib.type(n)
    else
      if n > 0 then
        1
      else if n < 0 then
        -1
      else 0,

  max(a, b)::
    if !stdlib.isNumber(a) then
      error 'stdlib.max first param expected number, got ' + stdlib.type(a)
    else if !stdlib.isNumber(b) then
      error 'stdlib.max second param expected number, got ' + stdlib.type(b)
    else
      if a > b then a else b,

  min(a, b)::
    if !stdlib.isNumber(a) then
      error 'stdlib.min first param expected number, got ' + stdlib.type(a)
    else if !stdlib.isNumber(b) then
      error 'stdlib.min second param expected number, got ' + stdlib.type(b)
    else
      if a < b then a else b,

  clamp(x, minVal, maxVal)::
    if x < minVal then minVal
    else if x > maxVal then maxVal
    else x,

  flattenArrays(arrs)::
    stdlib.foldl(function(a, b) a + b, arrs, []),

  flattenDeepArray(value)::
    if stdlib.isArray(value) then
      [y for x in value for y in stdlib.flattenDeepArray(x)]
    else
      [value],

  manifestIni(ini)::
    local body_lines(body) =
      stdlib.join([], [
        local value_or_values = body[k];
        if stdlib.isArray(value_or_values) then
          ['%s = %s' % [k, value] for value in value_or_values]
        else
          ['%s = %s' % [k, value_or_values]]

        for k in stdlib.objectFields(body)
      ]);

    local section_lines(sname, sbody) = ['[%s]' % [sname]] + body_lines(sbody),
          main_body = if stdlib.objectHas(ini, 'main') then body_lines(ini.main) else [],
          all_sections = [
      section_lines(k, ini.sections[k])
      for k in stdlib.objectFields(ini.sections)
    ];
    stdlib.join('\n', main_body + stdlib.flattenArrays(all_sections) + ['']),

  manifestToml(value):: stdlib.manifestTomlEx(value, '  '),

  manifestTomlEx(value, indent)::
    local
      escapeStringToml = stdlib.escapeStringJson,
      escapeKeyToml(key) =
        local bare_allowed = stdlib.set(stdlib.stringChars('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_-'));
        if stdlib.setUnion(stdlib.set(stdlib.stringChars(key)), bare_allowed) == bare_allowed then key else escapeStringToml(key),
      isTableArray(v) = stdlib.isArray(v) && stdlib.length(v) > 0 && stdlib.all(stdlib.map(stdlib.isObject, v)),
      isSection(v) = stdlib.isObject(v) || isTableArray(v),
      renderValue(v, indexedPath, inline, cindent) =
        if v == true then
          'true'
        else if v == false then
          'false'
        else if v == null then
          error 'Tried to manifest "null" at ' + indexedPath
        else if stdlib.isNumber(v) then
          '' + v
        else if stdlib.isString(v) then
          escapeStringToml(v)
        else if stdlib.isFunction(v) then
          error 'Tried to manifest function at ' + indexedPath
        else if stdlib.isArray(v) then
          if stdlib.length(v) == 0 then
            '[]'
          else
            local range = stdlib.range(0, stdlib.length(v) - 1);
            local new_indent = if inline then '' else cindent + indent;
            local separator = if inline then ' ' else '\n';
            local lines = ['[' + separator]
                          + stdlib.join([',' + separator],
                                        [
                                          [new_indent + renderValue(v[i], indexedPath + [i], true, '')]
                                          for i in range
                                        ])
                          + [separator + (if inline then '' else cindent) + ']'];
            stdlib.join('', lines)
        else if stdlib.isObject(v) then
          local lines = ['{ ']
                        + stdlib.join([', '],
                                      [
                                        [escapeKeyToml(k) + ' = ' + renderValue(v[k], indexedPath + [k], true, '')]
                                        for k in stdlib.objectFields(v)
                                      ])
                        + [' }'];
          stdlib.join('', lines),
      renderTableInternal(v, path, indexedPath, cindent) =
        local kvp = stdlib.flattenArrays([
          [cindent + escapeKeyToml(k) + ' = ' + renderValue(v[k], indexedPath + [k], false, cindent)]
          for k in stdlib.objectFields(v)
          if !isSection(v[k])
        ]);
        local sections = [stdlib.join('\n', kvp)] + [
          (
            if stdlib.isObject(v[k]) then
              renderTable(v[k], path + [k], indexedPath + [k], cindent)
            else
              renderTableArray(v[k], path + [k], indexedPath + [k], cindent)
          )
          for k in stdlib.objectFields(v)
          if isSection(v[k])
        ];
        stdlib.join('\n\n', sections),
      renderTable(v, path, indexedPath, cindent) =
        cindent + '[' + stdlib.join('.', stdlib.map(escapeKeyToml, path)) + ']'
        + (if v == {} then '' else '\n')
        + renderTableInternal(v, path, indexedPath, cindent + indent),
      renderTableArray(v, path, indexedPath, cindent) =
        local range = stdlib.range(0, stdlib.length(v) - 1);
        local sections = [
          (cindent + '[[' + stdlib.join('.', stdlib.map(escapeKeyToml, path)) + ']]'
           + (if v[i] == {} then '' else '\n')
           + renderTableInternal(v[i], path, indexedPath + [i], cindent + indent))
          for i in range
        ];
        stdlib.join('\n\n', sections);
    if stdlib.isObject(value) then
      renderTableInternal(value, [], [], '')
    else
      error 'TOML body must be an object. Got ' + stdlib.type(value),

  escapeStringJson(str_)::
    local str = stdlib.toString(str_);
    local trans(ch) =
      if ch == '"' then
        '\\"'
      else if ch == '\\' then
        '\\\\'
      else if ch == '\b' then
        '\\b'
      else if ch == '\f' then
        '\\f'
      else if ch == '\n' then
        '\\n'
      else if ch == '\r' then
        '\\r'
      else if ch == '\t' then
        '\\t'
      else
        local cp = stdlib.codepoint(ch);
        if cp < 32 || (cp >= 127 && cp <= 159) then
          '\\u%04x' % [cp]
        else
          ch;
    '"%s"' % stdlib.join('', [trans(ch) for ch in stdlib.stringChars(str)]),

  escapeStringPython(str)::
    stdlib.escapeStringJson(str),

  escapeStringBash(str_)::
    local str = stdlib.toString(str_);
    "'%s'" % stdlib.strReplace(str, "'", "'\"'\"'"),

  escapeStringDollars(str_)::
    local str = stdlib.toString(str_);
    stdlib.strReplace(str, '$', '$$'),

  local xml_escapes = {
    '<': '&lt;',
    '>': '&gt;',
    '&': '&amp;',
    '"': '&quot;',
    "'": '&apos;',
  },

  escapeStringXML(str_)::
    local str = stdlib.toString(str_);
    stdlib.join('', [stdlib.get(xml_escapes, ch, ch) for ch in stdlib.stringChars(str)]),

  manifestJson(value):: stdlib.manifestJsonEx(value, '    '),

  manifestJsonMinified(value):: stdlib.manifestJsonEx(value, '', '', ':'),

  manifestJsonEx(value, indent, newline='\n', key_val_sep=': ')::
    local aux(v, path, cindent) =
      if v == true then
        'true'
      else if v == false then
        'false'
      else if v == null then
        'null'
      else if stdlib.isNumber(v) then
        '' + v
      else if stdlib.isString(v) then
        stdlib.escapeStringJson(v)
      else if stdlib.isFunction(v) then
        error 'Tried to manifest function at ' + path
      else if stdlib.isArray(v) then
        local range = stdlib.range(0, stdlib.length(v) - 1);
        local new_indent = cindent + indent;
        local lines = ['[' + newline]
                      + stdlib.join([',' + newline],
                                    [
                                      [new_indent + aux(v[i], path + [i], new_indent)]
                                      for i in range
                                    ])
                      + [newline + cindent + ']'];
        stdlib.join('', lines)
      else if stdlib.isObject(v) then
        local lines = ['{' + newline]
                      + stdlib.join([',' + newline],
                                    [
                                      [cindent + indent + stdlib.escapeStringJson(k) + key_val_sep
                                       + aux(v[k], path + [k], cindent + indent)]
                                      for k in stdlib.objectFields(v)
                                    ])
                      + [newline + cindent + '}'];
        stdlib.join('', lines);
    aux(value, [], ''),

  manifestYamlDoc(value, indent_array_in_object=false, quote_keys=true)::
    local onlyChars(charSet, strSet) =
      if stdlib.length(stdlib.setInter(charSet, strSet)) == stdlib.length(strSet) then
        true
      else false;
    local isReserved(key) =
      // NOTE: These values are checked for case insensitively.
      // While this approach results in some false positives, it eliminates
      // the risk of missing a permutation.
      local reserved = [
        // Boolean types taken from https://yaml.org/type/bool.html
        'true',
        'false',
        'yes',
        'no',
        'on',
        'off',
        'y',
        'n',
        // Numerical words taken from https://yaml.org/type/float.html
        '.nan',
        '-.inf',
        '+.inf',
        '.inf',
        'null',
        // Invalid keys that contain no invalid characters
        '-',
        '---',
        '',
      ];
      local bad = [word for word in reserved if word == stdlib.asciiLower(key)];
      if stdlib.length(bad) > 0 then
        true
      else false;
    local typeMatch(m_key, type) =
      // Look for positive or negative numerical types (ex: 0x)
      if stdlib.substr(m_key, 0, 2) == type || stdlib.substr(m_key, 0, 3) == '-' + type then
        true
      else false;
    local bareSafe(key) =
      /*
      For a key to be considered safe to emit without quotes, the following must be true
        - All characters must match [a-zA-Z0-9_/\-]
        - Not match the integer format defined in https://yaml.org/type/int.html
        - Not match the float format defined in https://yaml.org/type/float.html
        - Not match the timestamp format defined in https://yaml.org/type/timestamp.html
        - Not match the boolean format defined in https://yaml.org/type/bool.html
        - Not match the null format defined in https://yaml.org/type/null.html
        - Not match (ignoring case) any reserved words which pass the above tests.
          Reserved words are defined in isReserved() above.

      Since the remaining YAML types require characters outside the set chosen as valid
      for the elimination of quotes from the YAML output, the remaining types listed at
      https://yaml.org/type/ are by default always quoted.
      */
      local letters = stdlib.set(stdlib.stringChars('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_-/'));
      local digits = stdlib.set(stdlib.stringChars('0123456789'));
      local intChars = stdlib.set(digits + stdlib.stringChars('_-'));
      local binChars = stdlib.set(intChars + stdlib.stringChars('b'));
      local hexChars = stdlib.set(digits + stdlib.stringChars('abcdefx_-'));
      local floatChars = stdlib.set(digits + stdlib.stringChars('e._-'));
      local dateChars = stdlib.set(digits + stdlib.stringChars('-'));
      local safeChars = stdlib.set(letters + floatChars);
      local keyLc = stdlib.asciiLower(key);
      local keyChars = stdlib.stringChars(key);
      local keySet = stdlib.set(keyChars);
      local keySetLc = stdlib.set(stdlib.stringChars(keyLc));
      // Check for unsafe characters
      if !onlyChars(safeChars, keySet) then
        false
      // Check for reserved words
      else if isReserved(key) then
        false
      /* Check for timestamp values.  Since spaces and colons are already forbidden,
         all that could potentially pass is the standard date format (ex MM-DD-YYYY, YYYY-DD-MM, etc).
         This check is even more conservative: Keys that meet all of the following:
           - all characters match [0-9\-]
           - has exactly 2 dashes
         are considered dates.
      */
      else if onlyChars(dateChars, keySet)
              && stdlib.length(stdlib.findSubstr('-', key)) == 2 then
        false
      /* Check for integers.  Keys that meet all of the following:
           - all characters match [0-9_\-]
           - has at most 1 dash
         are considered integers.
      */
      else if onlyChars(intChars, keySetLc)
              && stdlib.length(stdlib.findSubstr('-', key)) < 2 then
        false
      /* Check for binary integers.  Keys that meet all of the following:
           - all characters match [0-9b_\-]
           - has at least 3 characters
           - starts with (-)0b
         are considered binary integers.
      */
      else if onlyChars(binChars, keySetLc)
              && stdlib.length(key) > 2
              && typeMatch(key, '0b') then
        false
      /* Check for floats. Keys that meet all of the following:
           - all characters match [0-9e._\-]
           - has at most a single period
           - has at most two dashes
           - has at most 1 'e'
         are considered floats.
      */
      else if onlyChars(floatChars, keySetLc)
              && stdlib.length(stdlib.findSubstr('.', key)) == 1
              && stdlib.length(stdlib.findSubstr('-', key)) < 3
              && stdlib.length(stdlib.findSubstr('e', keyLc)) < 2 then
        false
      /* Check for hexadecimals.  Keys that meet all of the following:
           - all characters match [0-9a-fx_\-]
           - has at most 1 dash
           - has at least 3 characters
           - starts with (-)0x
         are considered hexadecimals.
      */
      else if onlyChars(hexChars, keySetLc)
              && stdlib.length(stdlib.findSubstr('-', key)) < 2
              && stdlib.length(keyChars) > 2
              && typeMatch(key, '0x') then
        false
      // All checks pass. Key is safe for emission without quotes.
      else true;
    local escapeKeyYaml(key) =
      if bareSafe(key) then key else stdlib.escapeStringJson(key);
    local aux(v, path, cindent) =
      if v == true then
        'true'
      else if v == false then
        'false'
      else if v == null then
        'null'
      else if stdlib.isNumber(v) then
        '' + v
      else if stdlib.isString(v) then
        local len = stdlib.length(v);
        if len == 0 then
          '""'
        else if v[len - 1] == '\n' then
          local split = stdlib.split(v, '\n');
          stdlib.join('\n' + cindent + '  ', ['|'] + split[0:stdlib.length(split) - 1])
        else
          stdlib.escapeStringJson(v)
      else if stdlib.isFunction(v) then
        error 'Tried to manifest function at ' + path
      else if stdlib.isArray(v) then
        if stdlib.length(v) == 0 then
          '[]'
        else
          local params(value) =
            if stdlib.isArray(value) && stdlib.length(value) > 0 then {
              // While we could avoid the new line, it yields YAML that is
              // hard to read, e.g.:
              // - - - 1
              //     - 2
              //   - - 3
              //     - 4
              new_indent: cindent + '  ',
              space: '\n' + self.new_indent,
            } else if stdlib.isObject(value) && stdlib.length(value) > 0 then {
              new_indent: cindent + '  ',
              // In this case we can start on the same line as the - because the indentation
              // matches up then.  The converse is not true, because fields are not always
              // 1 character long.
              space: ' ',
            } else {
              // In this case, new_indent is only used in the case of multi-line strings.
              new_indent: cindent,
              space: ' ',
            };
          local range = stdlib.range(0, stdlib.length(v) - 1);
          local parts = [
            '-' + param.space + aux(v[i], path + [i], param.new_indent)
            for i in range
            for param in [params(v[i])]
          ];
          stdlib.join('\n' + cindent, parts)
      else if stdlib.isObject(v) then
        if stdlib.length(v) == 0 then
          '{}'
        else
          local params(value) =
            if stdlib.isArray(value) && stdlib.length(value) > 0 then {
              // Not indenting allows e.g.
              // ports:
              // - 80
              // instead of
              // ports:
              //   - 80
              new_indent: if indent_array_in_object then cindent + '  ' else cindent,
              space: '\n' + self.new_indent,
            } else if stdlib.isObject(value) && stdlib.length(value) > 0 then {
              new_indent: cindent + '  ',
              space: '\n' + self.new_indent,
            } else {
              // In this case, new_indent is only used in the case of multi-line strings.
              new_indent: cindent,
              space: ' ',
            };
          local lines = [
            (if quote_keys then stdlib.escapeStringJson(k) else escapeKeyYaml(k)) + ':' + param.space + aux(v[k], path + [k], param.new_indent)
            for k in stdlib.objectFields(v)
            for param in [params(v[k])]
          ];
          stdlib.join('\n' + cindent, lines);
    aux(value, [], ''),

  manifestYamlStream(value, indent_array_in_object=false, c_document_end=true, quote_keys=true)::
    if !stdlib.isArray(value) then
      error 'manifestYamlStream only takes arrays, got ' + stdlib.type(value)
    else
      '---\n' + stdlib.join(
        '\n---\n', [stdlib.manifestYamlDoc(e, indent_array_in_object, quote_keys) for e in value]
      ) + if c_document_end then '\n...\n' else '\n',


  manifestPython(v)::
    if stdlib.isObject(v) then
      local fields = [
        '%s: %s' % [stdlib.escapeStringPython(k), stdlib.manifestPython(v[k])]
        for k in stdlib.objectFields(v)
      ];
      '{%s}' % [stdlib.join(', ', fields)]
    else if stdlib.isArray(v) then
      '[%s]' % [stdlib.join(', ', [stdlib.manifestPython(v2) for v2 in v])]
    else if stdlib.isString(v) then
      '%s' % [stdlib.escapeStringPython(v)]
    else if stdlib.isFunction(v) then
      error 'cannot manifest function'
    else if stdlib.isNumber(v) then
      stdlib.toString(v)
    else if v == true then
      'True'
    else if v == false then
      'False'
    else if v == null then
      'None',

  manifestPythonVars(conf)::
    local vars = ['%s = %s' % [k, stdlib.manifestPython(conf[k])] for k in stdlib.objectFields(conf)];
    stdlib.join('\n', vars + ['']),

  manifestXmlJsonml(value)::
    if !stdlib.isArray(value) then
      error 'Expected a JSONML value (an array), got %s' % stdlib.type(value)
    else
      local aux(v) =
        if stdlib.isString(v) then
          v
        else
          local tag = v[0];
          local has_attrs = stdlib.length(v) > 1 && stdlib.isObject(v[1]);
          local attrs = if has_attrs then v[1] else {};
          local children = if has_attrs then v[2:] else v[1:];
          local attrs_str =
            stdlib.join('', [' %s="%s"' % [k, attrs[k]] for k in stdlib.objectFields(attrs)]);
          stdlib.deepJoin(['<', tag, attrs_str, '>', [aux(x) for x in children], '</', tag, '>']);

      aux(value),

  local base64_table = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',
  local base64_inv = { [base64_table[i]]: i for i in stdlib.range(0, 63) },

  base64(input)::
    local bytes =
      if stdlib.isString(input) then
        stdlib.map(stdlib.codepoint, input)
      else
        input;

    local aux(arr, i, r) =
      if i >= stdlib.length(arr) then
        r
      else if i + 1 >= stdlib.length(arr) then
        local str =
          // 6 MSB of i
          base64_table[(arr[i] & 252) >> 2] +
          // 2 LSB of i
          base64_table[(arr[i] & 3) << 4] +
          '==';
        aux(arr, i + 3, r + str) tailstrict
      else if i + 2 >= stdlib.length(arr) then
        local str =
          // 6 MSB of i
          base64_table[(arr[i] & 252) >> 2] +
          // 2 LSB of i, 4 MSB of i+1
          base64_table[(arr[i] & 3) << 4 | (arr[i + 1] & 240) >> 4] +
          // 4 LSB of i+1
          base64_table[(arr[i + 1] & 15) << 2] +
          '=';
        aux(arr, i + 3, r + str) tailstrict
      else
        local str =
          // 6 MSB of i
          base64_table[(arr[i] & 252) >> 2] +
          // 2 LSB of i, 4 MSB of i+1
          base64_table[(arr[i] & 3) << 4 | (arr[i + 1] & 240) >> 4] +
          // 4 LSB of i+1, 2 MSB of i+2
          base64_table[(arr[i + 1] & 15) << 2 | (arr[i + 2] & 192) >> 6] +
          // 6 LSB of i+2
          base64_table[(arr[i + 2] & 63)];
        aux(arr, i + 3, r + str) tailstrict;

    local sanity = stdlib.all([a < 256 for a in bytes]);
    if !sanity then
      error 'Can only base64 encode strings / arrays of single bytes.'
    else
      aux(bytes, 0, ''),


  base64DecodeBytes(str)::
    if stdlib.length(str) % 4 != 0 then
      error 'Not a base64 encoded string "%s"' % str
    else
      local aux(str, i, r) =
        if i >= stdlib.length(str) then
          r
        else
          // all 6 bits of i, 2 MSB of i+1
          local n1 = [base64_inv[str[i]] << 2 | (base64_inv[str[i + 1]] >> 4)];
          // 4 LSB of i+1, 4MSB of i+2
          local n2 =
            if str[i + 2] == '=' then []
            else [(base64_inv[str[i + 1]] & 15) << 4 | (base64_inv[str[i + 2]] >> 2)];
          // 2 LSB of i+2, all 6 bits of i+3
          local n3 =
            if str[i + 3] == '=' then []
            else [(base64_inv[str[i + 2]] & 3) << 6 | base64_inv[str[i + 3]]];
          aux(str, i + 4, r + n1 + n2 + n3) tailstrict;
      aux(str, 0, []),

  base64Decode(str)::
    local bytes = stdlib.base64DecodeBytes(str);
    stdlib.join('', stdlib.map(stdlib.char, bytes)),

  reverse(arr)::
    local l = stdlib.length(arr);
    stdlib.makeArray(l, function(i) arr[l - i - 1]),

  // Merge-sort for long arrays and naive quicksort for shorter ones
  sort(arr, keyF=id)::
    local quickSort(arr, keyF=id) =
      local l = stdlib.length(arr);
      if stdlib.length(arr) <= 1 then
        arr
      else
        local pos = 0;
        local pivot = keyF(arr[pos]);
        local rest = stdlib.makeArray(l - 1, function(i) if i < pos then arr[i] else arr[i + 1]);
        local left = stdlib.filter(function(x) keyF(x) < pivot, rest);
        local right = stdlib.filter(function(x) keyF(x) >= pivot, rest);
        quickSort(left, keyF) + [arr[pos]] + quickSort(right, keyF);

    local merge(a, b) =
      local la = stdlib.length(a), lb = stdlib.length(b);
      local aux(i, j, prefix) =
        if i == la then
          prefix + b[j:]
        else if j == lb then
          prefix + a[i:]
        else
          if keyF(a[i]) <= keyF(b[j]) then
            aux(i + 1, j, prefix + [a[i]]) tailstrict
          else
            aux(i, j + 1, prefix + [b[j]]) tailstrict;
      aux(0, 0, []);

    local l = stdlib.length(arr);
    if stdlib.length(arr) <= 30 then
      quickSort(arr, keyF=keyF)
    else
      local mid = stdlib.floor(l / 2);
      local left = arr[:mid], right = arr[mid:];
      merge(stdlib.sort(left, keyF=keyF), stdlib.sort(right, keyF=keyF)),

  uniq(arr, keyF=id)::
    local f(a, b) =
      if stdlib.length(a) == 0 then
        [b]
      else if keyF(a[stdlib.length(a) - 1]) == keyF(b) then
        a
      else
        a + [b];
    stdlib.foldl(f, arr, []),

  set(arr, keyF=id)::
    stdlib.uniq(stdlib.sort(arr, keyF), keyF),

  setMember(x, arr, keyF=id)::
    // TODO(dcunnin): Binary chop for O(log n) complexity
    stdlib.length(stdlib.setInter([x], arr, keyF)) > 0,

  setUnion(a, b, keyF=id)::
    // NOTE: order matters, values in `a` win
    local aux(a, b, i, j, acc) =
      if i >= stdlib.length(a) then
        acc + b[j:]
      else if j >= stdlib.length(b) then
        acc + a[i:]
      else
        local ak = keyF(a[i]);
        local bk = keyF(b[j]);
        if ak == bk then
          aux(a, b, i + 1, j + 1, acc + [a[i]]) tailstrict
        else if ak < bk then
          aux(a, b, i + 1, j, acc + [a[i]]) tailstrict
        else
          aux(a, b, i, j + 1, acc + [b[j]]) tailstrict;
    aux(a, b, 0, 0, []),

  setInter(a, b, keyF=id)::
    local aux(a, b, i, j, acc) =
      if i >= stdlib.length(a) || j >= stdlib.length(b) then
        acc
      else
        if keyF(a[i]) == keyF(b[j]) then
          aux(a, b, i + 1, j + 1, acc + [a[i]]) tailstrict
        else if keyF(a[i]) < keyF(b[j]) then
          aux(a, b, i + 1, j, acc) tailstrict
        else
          aux(a, b, i, j + 1, acc) tailstrict;
    aux(a, b, 0, 0, []) tailstrict,

  setDiff(a, b, keyF=id)::
    local aux(a, b, i, j, acc) =
      if i >= stdlib.length(a) then
        acc
      else if j >= stdlib.length(b) then
        acc + a[i:]
      else
        if keyF(a[i]) == keyF(b[j]) then
          aux(a, b, i + 1, j + 1, acc) tailstrict
        else if keyF(a[i]) < keyF(b[j]) then
          aux(a, b, i + 1, j, acc + [a[i]]) tailstrict
        else
          aux(a, b, i, j + 1, acc) tailstrict;
    aux(a, b, 0, 0, []) tailstrict,

  mergePatch(target, patch)::
    if stdlib.isObject(patch) then
      local target_object =
        if stdlib.isObject(target) then target else {};

      local target_fields =
        if stdlib.isObject(target_object) then stdlib.objectFields(target_object) else [];

      local null_fields = [k for k in stdlib.objectFields(patch) if patch[k] == null];
      local both_fields = stdlib.setUnion(target_fields, stdlib.objectFields(patch));

      {
        [k]:
          if !stdlib.objectHas(patch, k) then
            target_object[k]
          else if !stdlib.objectHas(target_object, k) then
            stdlib.mergePatch(null, patch[k]) tailstrict
          else
            stdlib.mergePatch(target_object[k], patch[k]) tailstrict
        for k in stdlib.setDiff(both_fields, null_fields)
      }
    else
      patch,

  get(o, f, default=null, inc_hidden=true)::
    if std.objectHasEx(o, f, inc_hidden) then o[f] else default,

  objectFields(o)::
    stdlib.objectFieldsEx(o, false),

  objectFieldsAll(o)::
    stdlib.objectFieldsEx(o, true),

  objectHas(o, f)::
    stdlib.objectHasEx(o, f, false),

  objectHasAll(o, f)::
    stdlib.objectHasEx(o, f, true),

  objectValues(o)::
    [o[k] for k in stdlib.objectFields(o)],

  objectValuesAll(o)::
    [o[k] for k in stdlib.objectFieldsAll(o)],

  objectKeysValues(o)::
    [{ key: k, value: o[k] } for k in stdlib.objectFields(o)],

  objectKeysValuesAll(o)::
    [{ key: k, value: o[k] } for k in stdlib.objectFieldsAll(o)],

  equals(a, b)::
    local ta = stdlib.type(a);
    local tb = stdlib.type(b);
    if !stdlib.primitiveEquals(ta, tb) then
      false
    else
      if stdlib.primitiveEquals(ta, 'array') then
        local la = stdlib.length(a);
        if !stdlib.primitiveEquals(la, stdlib.length(b)) then
          false
        else
          local aux(a, b, i) =
            if i >= la then
              true
            else if a[i] != b[i] then
              false
            else
              aux(a, b, i + 1) tailstrict;
          aux(a, b, 0)
      else if stdlib.primitiveEquals(ta, 'object') then
        local fields = stdlib.objectFields(a);
        local lfields = stdlib.length(fields);
        if fields != stdlib.objectFields(b) then
          false
        else
          local aux(a, b, i) =
            if i >= lfields then
              true
            else if local f = fields[i]; a[f] != b[f] then
              false
            else
              aux(a, b, i + 1) tailstrict;
          aux(a, b, 0)
      else
        stdlib.primitiveEquals(a, b),


  resolvePath(f, r)::
    local arr = stdlib.split(f, '/');
    stdlib.join('/', stdlib.makeArray(stdlib.length(arr) - 1, function(i) arr[i]) + [r]),

  prune(a)::
    local isContent(b) =
      if b == null then
        false
      else if stdlib.isArray(b) then
        stdlib.length(b) > 0
      else if stdlib.isObject(b) then
        stdlib.length(b) > 0
      else
        true;
    if stdlib.isArray(a) then
      [stdlib.prune(x) for x in a if isContent($.prune(x))]
    else if stdlib.isObject(a) then {
      [x]: $.prune(a[x])
      for x in stdlib.objectFields(a)
      if isContent(stdlib.prune(a[x]))
    } else
      a,

  findSubstr(pat, str)::
    if !stdlib.isString(pat) then
      error 'findSubstr first parameter should be a string, got ' + stdlib.type(pat)
    else if !stdlib.isString(str) then
      error 'findSubstr second parameter should be a string, got ' + stdlib.type(str)
    else
      local pat_len = stdlib.length(pat);
      local str_len = stdlib.length(str);
      if pat_len == 0 || str_len == 0 || pat_len > str_len then
        []
      else
        stdlib.filter(function(i) str[i:i + pat_len] == pat, stdlib.range(0, str_len - pat_len)),

  find(value, arr)::
    if !stdlib.isArray(arr) then
      error 'find second parameter should be an array, got ' + stdlib.type(arr)
    else
      stdlib.filter(function(i) arr[i] == value, stdlib.range(0, stdlib.length(arr) - 1)),

  all(arr)::
    assert stdlib.isArray(arr) : 'all() parameter should be an array, got ' + stdlib.type(arr);
    local arrLen = stdlib.length(arr);
    local aux(idx) =
      if idx >= arrLen then
        true
      else
        local e = arr[idx];
        assert stdlib.isBoolean(e) : 'element "%s" of type %s is not a boolean' % [e, stdlib.type(e)];
        if !e then
          false
        else
          aux(idx + 1) tailstrict;
    aux(0),

  any(arr)::
    assert stdlib.isArray(arr) : 'any() parameter should be an array, got ' + stdlib.type(arr);
    local arrLen = stdlib.length(arr);
    local aux(idx) =
      if idx >= arrLen then
        false
      else
        local e = arr[idx];
        assert stdlib.isBoolean(e) : 'element "%s" of type %s is not a boolean' % [e, stdlib.type(e)];
        if e then
          true
        else
          aux(idx + 1) tailstrict;
    aux(0),

  // Three way comparison.
  // TODO(sbarzowski): consider exposing and documenting it properly
  __compare(v1, v2)::
    local t1 = stdlib.type(v1), t2 = stdlib.type(v2);
    if t1 != t2 then
      error 'Comparison requires matching types. Got ' + t1 + ' and ' + t2
    else if t1 == 'array' then
      stdlib.__compare_array(v1, v2)
    else if t1 == 'function' || t1 == 'object' || t1 == 'boolean' then
      error 'Values of type ' + t1 + ' are not comparable.'
    else if v1 < v2 then -1
    else if v1 > v2 then 1
    else 0,

  __compare_array(arr1, arr2)::
    local len1 = stdlib.length(arr1), len2 = stdlib.length(arr2);
    local minLen = stdlib.min(len1, len2);
    local aux(i) =
      if i < minLen then
        local cmpRes = stdlib.__compare(arr1[i], arr2[i]);
        if cmpRes != 0 then
          cmpRes
        else
          aux(i + 1) tailstrict
      else
        stdlib.__compare(len1, len2);
    aux(0),

  __array_less(arr1, arr2):: stdlib.__compare_array(arr1, arr2) == -1,
  __array_greater(arr1, arr2):: stdlib.__compare_array(arr1, arr2) == 1,
  __array_less_or_equal(arr1, arr2):: stdlib.__compare_array(arr1, arr2) <= 0,
  __array_greater_or_equal(arr1, arr2):: stdlib.__compare_array(arr1, arr2) >= 0,

  sum(arr):: stdlib.foldl(function(a, b) a + b, arr, 0),

  avg(arr)::
    if stdlib.length(arr) == 0 then
      error 'Cannot calculate average of an empty array.'
    else
      stdlib.sum(arr) / stdlib.length(arr),

  minArray(arr, keyF=id, onEmpty=error 'Expected at least one element in array. Got none')::
    if stdlib.length(arr) == 0 then
      onEmpty
    else
      local minVal = arr[0];
      local minFn(a, b) =
        if stdlib.__compare(keyF(a), keyF(b)) > 0 then
          b
        else
          a;
      stdlib.foldl(minFn, arr, minVal),

  maxArray(arr, keyF=id, onEmpty=error 'Expected at least one element in array. Got none')::
    if stdlib.length(arr) == 0 then
      onEmpty
    else
      local maxVal = arr[0];
      local maxFn(a, b) =
        if stdlib.__compare(keyF(a), keyF(b)) < 0 then
          b
        else
          a;
      stdlib.foldl(maxFn, arr, maxVal),

  xor(x, y):: x != y,

  xnor(x, y):: x == y,

  round(x):: stdlib.floor(x + 0.5),

  isEmpty(str):: stdlib.length(str) == 0,

  contains(arr, elem):: stdlib.any([e == elem for e in arr]),

  equalsIgnoreCase(str1, str2):: stdlib.asciiLower(str1) == stdlib.asciiLower(str2),

  isEven(x):: stdlib.round(x) % 2 == 0,
  isOdd(x):: stdlib.round(x) % 2 != 0,
  isInteger(x):: stdlib.round(x) == x,
  isDecimal(x):: stdlib.round(x) != x,

  removeAt(arr, at):: [
    arr[i]
    for i in stdlib.range(0, stdlib.length(arr) - 1)
    if i != at
  ],

  remove(arr, elem)::
    local indexes = stdlib.find(elem, arr);
    if stdlib.length(indexes) == 0
    then
      arr
    else
      stdlib.removeAt(arr, indexes[0])
  ,

  objectRemoveKey(obj, key):: {
    [k]: obj[k]
    for k in stdlib.objectFields(obj)
    if k != key
  },

  sha1(str):: go_only_function,
  sha256(str):: go_only_function,
  sha512(str):: go_only_function,
  sha3(str):: go_only_function,

  trim(str):: stdlib.stripChars(str, ' \t\n\f\r\u0085'),
}
