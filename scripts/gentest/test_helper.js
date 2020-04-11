function define_element_prop(property, getter) {
  if (!(property in Element.prototype)) {
    Object.defineProperty(Element.prototype, property, {
      get: function() {
        return getter(this);
      }
    });
  }
}

define_element_prop("__stretch_description__", (e) => {
  return JSON.stringify(describeElement(e));
});

function parseGridArea(input) {
  if (!input) {
    return undefined;
  }
  if (input === 'auto') {
    return {kind: 'auto'}
  } else {
    return {kind: 'explicit', value: input.split('/').map(t => parseInt(t, 10))};
  }
}

// https://www.w3.org/TR/css-grid-1/#line-placement
// <grid-line> = 
//   auto |
//   <custom-ident> |
//   [ <integer> && <custom-ident>? ] |
//   [ span && [ <integer> || <custom-ident> ] ]
function parseGridLine (input) {
  if (!input) {
    return undefined;
  }
  
  const n = parseInt(input, 10)
  return {kind: 'nth', value: isNaN(n) ? 1 : n}
}

function parseTrackList(input) {
  // TODO support repeat: https://www.w3.org/TR/css-grid-1/#typedef-auto-track-list
  // TODO support explicit line names
  if (!input) {
    return undefined;
  }
  const segments = input.split(' ').filter(v => v).reduce(({current, values}, str) => {
    const next = current+str;
    if (next.split('(').length === next.split(')').length) { // matched parens
      return {current: '', values: [...values, next]}
    } else {
      return {current:next, values}
    }
  }, {current: '', values: []}).values;
  
  return segments.map(v => parseGridTemplatesValue(v));
}

function parseGridTemplatesValueHelper(input) {
  const [unit, ...rest] = input.split(/[(,)]/g).filter(v => v)
  const value = rest.map(v => parseGridTemplatesValueHelper(v))
  switch (unit) {
    case 'auto':
    case 'min-content':
    case 'max-content':
      return {unit};
    case 'minmax':
      return {unit, value};
    case 'fit-content':
      return {unit: 'minmax', value:[{unit:'auto'},{...values[0],unit: 'auto-capped'}]};
    default:
      if (unit.endsWith("px")) {
        return {
          unit: 'points', 
          value: Number(unit.replace('px',''))
        };
      } else if (unit.endsWith("%")) {
        return {
          unit: 'percent', 
          value: Number(unit.replace('%','')) / 100
        };
      } else if (unit.endsWith("fr")) {
        return {
          unit: 'flex', 
          value: Number(unit.replace('fr','')) 
        };
      }
  }
}

function parseGridTemplatesValue(input) {
  parsed = parseGridTemplatesValueHelper(input);
  if (parsed.unit === 'minmax') {
    return parsed.value;
  } else {
    return [parsed, parsed];
  }
}

function parseGridTemplates(input) {
  if (!input) {
    return undefined;
  }
  const segments = input.split(' ').filter(v => v).reduce(({current, values}, str) => {
    const next = current+str;
    if (next.split('(').length === next.split(')').length) { // matched parens
      return {current: '', values: [...values, next]}
    } else {
      return {current:next, values}
    }
  }, {current: '', values: []}).values;
  let fill = 'auto';
  if (segments[segments.length -1] === 'auto-repeat') {
    fill = segments[segments.length - 2]
    segments.pop();
    segments.pop();
  }
  return segments.map(v => parseGridTemplatesValue(v)).flat().concat(parseGridTemplatesValue(fill));
}

function parseDimension(input) {
  if (input.endsWith("px")) {
    return {
      unit: 'points', 
      value: Number(input.replace('px',''))
    };
  } else if (input.endsWith("%")) {
    return {
      unit: 'percent', 
      value: Number(input.replace('%','')) / 100
    };
  } else {
    return input == "auto" ? {unit: "auto"} : undefined;
  }
}

function parseNumber(input) {
  if (input === "" || isNaN(input)) {
    return undefined;
  } else {
    return Number(input);
  }
}

function parseEnum(input) {
  if (input) {
    return input;
  } else {
    return undefined;
  }
}

function parseEdges(edges) {
  var start = parseDimension(edges.start);
  var end = parseDimension(edges.end);
  var top = parseDimension(edges.top);
  var bottom = parseDimension(edges.bottom);
  
  if (start === undefined && end === undefined && top === undefined && bottom === undefined) {
    return undefined;
  }

  return {
    start: start,
    end: end,
    top: top,
    bottom: bottom
  };
}

function parseSize(size) {
  var width = parseDimension(size.width);
  var height = parseDimension(size.height);
  
  if (width === undefined && height === undefined) {
    return undefined;
  }

  return {
    width: width,
    height: height,
  };
}

function describeElement(e) {
  return {
    style: {
      display: parseEnum(e.style.display),

      position_type: parseEnum(e.style.position),
      direction: parseEnum(e.style.direction),
      flexDirection: parseEnum(e.style.flexDirection),

      flexWrap: parseEnum(e.style.flexWrap),
      overflow: parseEnum(e.style.overflow),

      alignItems: parseEnum(e.style.alignItems),
      alignSelf: parseEnum(e.style.alignSelf),
      alignContent: parseEnum(e.style.alignContent),
      
      justifyContent: parseEnum(e.style.justifyContent),

      flexGrow: parseNumber(e.style.flexGrow),
      flexShrink: parseNumber(e.style.flexShrink),
      flexBasis: parseDimension(e.style.flexBasis),

      gridArea: parseGridArea(e.style.gridArea),
      gridRows: parseGridTemplates(e.style.gridTemplateRows),
      gridColumns: parseGridTemplates(e.style.gridTemplateColumns),
      gridGap: parseSize({width: e.style.columnGap, height: e.style.rowGap}),
      
      gridTemplateRowBounds: parseTrackList(e.style.gridTemplateRows),
      gridTemplateColumnBounds: parseTrackList(e.style.gridTemplateColumns),
      gridRowStart: parseGridLine(e.style.gridRowStart),
      gridRowEnd: parseGridLine(e.style.gridRowEnd),
      gridColumnStart: parseGridLine(e.style.gridColumnStart),
      gridColumnEnd: parseGridLine(e.style.gridColumnEnd),
      
      size: parseSize({width: e.style.width, height: e.style.height}),
      min_size: parseSize({width: e.style.minWidth, height: e.style.minHeight}),
      max_size: parseSize({width: e.style.maxWidth, height: e.style.maxHeight}),

      margin: parseEdges({
        start: e.style.marginLeft,
        end: e.style.marginRight,
        top: e.style.marginTop,
        bottom: e.style.marginBottom,
      }),

      padding: parseEdges({
        start: e.style.paddingLeft,
        end: e.style.paddingRight,
        top: e.style.paddingTop,
        bottom: e.style.paddingBottom,
      }),

      border: parseEdges({
        start: e.style.borderLeftWidth,
        end: e.style.borderRightWidth,
        top: e.style.borderTopWidth,
        bottom: e.style.borderBottomWidth,
      }),

      position: parseEdges({
        start: e.style.left,
        end: e.style.right,
        top: e.style.top,
        bottom: e.style.bottom,
      }),
    },

    layout: {
      width: e.offsetWidth,
      height: e.offsetHeight,
      x: e.offsetLeft + e.parentNode.clientLeft,
      y: e.offsetTop + e.parentNode.clientTop,
    },

    children: Array.from(e.children).map(c => describeElement(c)),
  }
}
