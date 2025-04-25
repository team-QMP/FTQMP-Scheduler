from solid.objects import cube, union, translate, color

def create_polycube(coords, polycubecolor = False, unit_size=1):
    """
    Generate a polycube from a list of coordinates.
    :param coords: List of coordinates for cube placement [(x, y, z), ...]
    :param unit_size: Size of each cube (default 10mm)
    :return: 3D object of the polycube
    """
    if polycubecolor:
        cubes = [color(polycubecolor)(translate((x * unit_size, y * unit_size, z * unit_size)))(cube(unit_size)) for x, y, z in coords]
    else:
        cubes = [translate((x * unit_size, y * unit_size, z * unit_size))(cube(unit_size)) for x, y, z in coords]
    print("cubes", cubes)
    return union()(*cubes)

def create_scheduled_polycube(coords, origin, polycubecolor = False, unit_size=1):
    """
    Generate a polycube with a specified origin.
    :param coords: List of coordinates for cube placement [(x, y, z), ...]
    :param origin: Origin point for the polycube
    :param: polycubecolor: color of the polycube
    :param unit_size: Size of each cube (default 10mm)
    :return: 3D object of the polycube
    """
    if polycubecolor:
        cubes = [color(polycubecolor)(translate((x * unit_size + origin[0], y * unit_size+ origin[1], z * unit_size+ origin[2]))(cube(unit_size))) for x, y, z in coords]
    else:
        cubes = [translate((x * unit_size + origin[0], y * unit_size + origin[1], z * unit_size+ origin[2]))(cube(unit_size)) for x, y, z in coords]
    return union()(*cubes)

def create_polycubes(coords_list, colors, unit_size=1):
    """
    Generate multiple polycubes from a list of coordinate lists.
    :param coords_list: List of lists of coordinates for cube placement [[(x, y, z), ...], ...]
    :param colors: List of colors for each polycube ["red", "green", ...]
    :param unit_size: Size of each cube (default 10mm)
    """
    polycubes = []
    for i, coords in enumerate(coords_list):
        color = colors[i % len(colors)]
        polycube_model = create_scheduled_polycube(coords, origin=(0, 0, 0), polycubecolor=color, unit_size=unit_size)
        polycubes.append(polycube_model)
    return union()(*polycubes)