import mathutils
import bpy

#bpy.ops.wm.open_mainfile()

def unselect_all():
    for ob in bpy.context.selected_objects:
        ob.select_set(False)

def bounding_box_corners(ob_name: str) -> list[mathutils.Vector]:
    ob = bpy.data.objects[ob_name]
    return [ob.matrix_world @ mathutils.Vector(corner) for corner in ob.bound_box]

def bounding_box_center(ob_name: str) -> mathutils.Vector:
    result = mathutils.Vector((0.0, 0.0, 0.0))
    for corner in bounding_box_corners(ob_name):
        result = result + corner
    return result / 8.0

def shrink_wrap(target_name: str) -> str:
    target = bpy.data.objects[target_name]
    unselect_all()

    wrapper_name = "Bake" + target_name + "Wrapper"
    if not (bpy.data.objects.get(wrapper_name) is None):
        bpy.data.objects.remove(bpy.data.objects[wrapper_name])

    bpy.ops.mesh.primitive_cube_add()
    bpy.context.active_object.name = wrapper_name

    corners = bounding_box_corners(target_name)
    max_dim = 0
    for c1 in corners:
        for c2 in corners:
            max_dim = max(max_dim, (c1 - c2).length)
    scale = max_dim * 1.25 / 2.0

    bpy.ops.transform.translate(value=bounding_box_center(target_name))
    bpy.ops.transform.resize(value=(scale, scale, scale))
    
    subdivision_levels = 4
    offset = 0.5

    bpy.ops.object.modifier_add(type="SUBSURF")
    bpy.context.active_object.modifiers["Subdivision"].subdivision_type = "SIMPLE"
    bpy.context.active_object.modifiers["Subdivision"].levels = subdivision_levels
    bpy.context.active_object.modifiers["Subdivision"].render_levels = subdivision_levels
    bpy.ops.object.modifier_apply(modifier="Subdivision")
    
    bpy.ops.object.modifier_add(type="SHRINKWRAP")
    bpy.context.active_object.modifiers["Shrinkwrap"].target = target
    bpy.context.active_object.modifiers["Shrinkwrap"].offset = offset
    bpy.ops.object.modifier_apply(modifier="Shrinkwrap")
    
    bpy.ops.object.modifier_add(type="SMOOTH")
    bpy.context.active_object.modifiers["Smooth"].iterations = 20
    bpy.ops.object.modifier_apply(modifier="Smooth")
    
    bpy.ops.object.editmode_toggle()
    bpy.ops.uv.select_all()
    #bpy.ops.uv.smart_project(angle_limit=1.15192, island_margin=0.01, area_weight=0, correct_aspect=True, scale_to_bounds=True)
    #bpy.ops.uv.minimize_stretch(iterations=5000)
    bpy.ops.uv.lightmap_pack(PREF_IMG_PX_SIZE=1536)
    bpy.ops.object.editmode_toggle()

    return wrapper_name

def create_image(image_name: str):
    # Delete image if it already exists
    if not (bpy.data.images.get(image_name) is None):
        bpy.data.images.remove(bpy.data.images[image_name])
    bpy.ops.image.new(name=image_name, float=True, width=1536, height=1536)

def set_up_texture_image_node(node_name: str, material_name: str, image_name: str):
    image = bpy.data.images[image_name]
    material = bpy.data.materials[material_name]
    # Delete bake node if it already exists
    if not (material.node_tree.nodes.get(node_name) is None):
        material.node_tree.nodes.remove(material.node_tree.nodes[node_name])
    material.node_tree.nodes.active = None
    node = material.node_tree.nodes.new(type="ShaderNodeTexImage")
    node.name = node_name
    node.image = image
    material.node_tree.nodes.active = node

def set_up_texture_nodes(name: str, target_name: str, wrapper_name: str):
    # Create the bake image
    image_name = "Bake" + target_name + name + "Image"
    create_image(image_name)
    
    # Add bake nodes to each material on the target object
    for material_name in bpy.data.objects[target_name].material_slots.keys():
        node_name = "Bake" + material_name + "Node"
        set_up_texture_image_node(node_name, material_name, image_name)
    
    wrapper_node_name = "Bake" + target_name + name + "WrapperNode"
    wrapper_material_name = bpy.data.objects[wrapper_name].material_slots[0].name
    set_up_texture_image_node(wrapper_node_name, wrapper_material_name, image_name)
    
    return wrapper_node_name

def bake_normal_maps(target_name: str):    
    #wrapper_name = shrink_wrap(target_name)
    wrapper_name = "Bake" + target_name + "Wrapper"
    
    # Make sure the wrapper has no material slots
    bpy.data.objects[wrapper_name].data.materials.clear()
    
    # Create the wrapper material
    wrapper_material_name = "Bake" + target_name + "WrapperMaterial"
    if not (bpy.data.materials.get(wrapper_material_name) is None):
        bpy.data.materials.remove(bpy.data.materials[wrapper_material_name])
    wrapper_material = bpy.data.materials.new(name=wrapper_material_name)
    wrapper_material.use_nodes = True
    bpy.data.objects[wrapper_name].data.materials.append(wrapper_material)
    
    cage_extrusion = 0.0
    max_ray_distance = 4.0
    
    normal_node_name = set_up_texture_nodes("NormalMap", target_name, wrapper_name)
    unselect_all()
    bpy.data.objects[target_name].select_set(True)
    bpy.data.objects[wrapper_name].select_set(True)
    bpy.context.view_layer.objects.active = bpy.data.objects[wrapper_name]
    bpy.ops.object.bake(
        type="NORMAL",
        use_selected_to_active=True,
        use_clear=True,
        cage_extrusion=cage_extrusion,
        max_ray_distance=max_ray_distance)
    unselect_all()

    combined_node_name = set_up_texture_nodes("CombinedMap", target_name, wrapper_name)
    unselect_all()
    bpy.data.objects[target_name].select_set(True)
    bpy.data.objects[wrapper_name].select_set(True)
    bpy.context.view_layer.objects.active = bpy.data.objects[wrapper_name]
    bpy.ops.object.bake(
        type="COMBINED",
        use_selected_to_active=True,
        use_clear=True,
        cage_extrusion=cage_extrusion,
        max_ray_distance=max_ray_distance)
    unselect_all()

    nodes = bpy.data.materials[wrapper_material_name].node_tree.nodes
    links = bpy.data.materials[wrapper_material_name].node_tree.links
    nodes.new(type="ShaderNodeNormalMap")
    links.new(nodes[normal_node_name].outputs["Color"], nodes["Normal Map"].inputs["Color"])
    links.new(nodes[combined_node_name].outputs["Color"], nodes["Principled BSDF"].inputs["Base Color"])
    links.new(nodes["Normal Map"].outputs["Normal"], nodes["Principled BSDF"].inputs["Normal"])

def rerun():
    bpy.data.texts["asset_pipeline"].as_module().bake_normal_maps("Segments")
   
    #target_node_name = "Bake" + target_name + "NormalMapNode"
    #wrapper_node_name 
#bpy.context.active_object.material_slots['DarkShellSegment'].material.node_tree.nodes.new(type="ShaderNodeTexImage")
#bpy.context.active_object.material_slots['DarkShellSegment'].material.node_tree.nodes.remove("SegmentsNormalMap")
#dark_shell_segment.node_tree.nodes.remove(dark_shell_segment.node_tree.nodes["Image Texture"])
# dark_shell_segment.node_tree.nodes.get("Principled BSDF ") is None
#bpy.context.active_object.material_slots['DarkShellSegment'].material.node_tree.nodes['Image Texture'].image = bpy.data.images['SegmentsNormalMap']
#bpy.ops.image.new(name="Foo")

# asset_pipeline = bpy.data.texts['asset_pipeline'].as_module()
# asset_pipeline.shrink_wrap(bpy.context.active_object.name)


#bpy.context.scene.render.use_bake_multires = False
#bpy.context.scene.cycles.bake_type = "COMBINED"
#bpy.context.scene.cycles.bake_type = "NORMAL"
#bpy.context.scene.render.bake.normal_space = 'TANGENT'
#bpy.context.scene.render.bake.normal_r = 'POS_X'
#bpy.context.scene.render.bake.normal_g = 'POS_Y'
#bpy.context.scene.render.bake.normal_b = 'POS_Z'
#bpy.context.scene.render.bake.use_selected_to_active = True
#bpy.context.scene.render.bake.use_cage = False
#bpy.context.scene.render.bake.cage_extrusion = 2
#bpy.context.scene.render.bake.max_ray_distance = 0
#bpy.context.scene.render.bake.target = 'IMAGE_TEXTURES'
#bpy.context.scene.render.bake.use_clear = True
#bpy.context.scene.render.bake.margin = 16

#bpy.ops.object.bake(type='COMBINED', save_mode='EXTERNAL')
#bpy.ops.object.bake(type='COMBINED', pass_filter={}, filepath='', width=512, height=512, margin=16, margin_type='EXTEND', use_selected_to_active=False, max_ray_distance=0.0, cage_extrusion=0.0, cage_object='', normal_space='TANGENT', normal_r='POS_X', normal_g='POS_Y', normal_b='POS_Z', target='IMAGE_TEXTURES', save_mode='INTERNAL', use_clear=False, use_cage=False, use_split_materials=False, use_automatic_name=False, uv_layer='')