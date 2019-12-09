'use strict';

/* XXX: figure out ts-loader */
import { HexGridPicker } from "!ts-loader!./hexgrid-utils.ts";
import { initial_battle } from "hexgrid";
import { Picker } from "!ts-loader!./mouse-canvas-picker.ts";
import * as THREE from "three";
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';


const SQRT3 = Math.sqrt(3);
const HALF_SQRT3 = SQRT3 / 2;
const HEX_COLOR = 0x004400;
const SELECTED_HEX_COLOR = 0x330000;

var debug = true;

var battle;
var camera;
var canvas;
var controls;
var grid;
var renderer;
var rootObject = new THREE.Object3D();
var scene = new THREE.Scene();

var picker;
var hexGridPicker = new HexGridPicker();

function onWindowResize() {
    const width = canvas.clientWidth;
    const height = canvas.clientHeight;

    if (canvas.width === width && canvas.height === height) {
        return;
    }

    camera.aspect = width / height;
    camera.updateProjectionMatrix();

    /* apparently this can be expensive, so always check if needed first */
    renderer.setSize(width, height, false);
}

function hexShape(size) {
    const path = new THREE.Shape();
    path.moveTo(size * 0,           size * 1);
    path.lineTo(size * HALF_SQRT3,  size * 0.5);
    path.lineTo(size * HALF_SQRT3,  size * -0.5);
    path.lineTo(size * 0,           size * -1);
    path.lineTo(size * -HALF_SQRT3, size * -0.5);
    path.lineTo(size * -HALF_SQRT3, size * 0.5);
    path.lineTo(size * 0,           size * 1);

    const extrudeSettings = {
        steps: 1,
        depth: 3,
        bevelEnabled: false,
        bevelThickness: 0,
        bevelSize: 0,
        bevelSegments: 0,
    };
    return new THREE.ExtrudeBufferGeometry(path, extrudeSettings);
}

/* convert to three.js coords assuming q=0, r=0 is at (0, 0, 0) */
function axialToPosition(coord, radius) {
    const {q, r} = coord;
    return {
        x: radius * ((SQRT3 * q) + (HALF_SQRT3 * r)),
        y: radius * r * 3 / 2,
    }
}

function addHexGeometry(geometry, coord, radius) {
    // const material = new THREE.MeshBasicMaterial({color: 0xaa00ff});
    const material = new THREE.MeshPhongMaterial({
        side: THREE.DoubleSide,
        color: HEX_COLOR,
    });
    /* add 0.1 for a gap between the hexes */
    const position = axialToPosition(coord, radius + 0.1);

    const mesh = new THREE.Mesh(geometry, material);

    if (false) {
      /* XXX: translating these things doesn't seem to work */
      const axes = new THREE.AxesHelper();
      axes.material.depthTest = false;
      axes.renderOrder = 2;
      mesh.add(axes);

      const helper = new THREE.GridHelper(5, 5);
      helper.material.depthTest = false;
      helper.renderOrder = 1;
      mesh.add(helper);
    }

    mesh.position.set(position.x, position.y, 0);
    rootObject.add(mesh);

    /* so we can change the mesh later */
    hexGridPicker.addHexMesh(coord, mesh);

    /*
     * apparently a custom shader would be better because there wouldn't have
     * to be any light calculations for this material
     * https://threejsfundamentals.org/threejs/lessons/threejs-picking.html
     * XXX: not sure why you overload emissive instead of color
     */
    const pickingMaterial = new THREE.MeshPhongMaterial({
        emissive: HexGridPicker.axialCoordToPickerID(coord),
        color: new THREE.Color(0, 0, 0),
        specular: new THREE.Color(0, 0, 0),
        map: null,
        transparent: true,
        side: THREE.DoubleSide,
        alphaTest: 0.5,
        blending: THREE.NoBlending,
    });
    const pickerMesh = new THREE.Mesh(geometry, pickingMaterial);
    pickerMesh.position.copy(mesh.position);
    pickerMesh.rotation.copy(mesh.rotation);
    pickerMesh.scale.copy(mesh.scale);
    picker.rootObject.add(pickerMesh);
}

function addLights(scene) {
	scene.add(new THREE.HemisphereLight());

    {
        const color = 0xFFFFFF;
        const intensity = 1;
        const light = new THREE.DirectionalLight(color, intensity);
        light.position.set(-1, 2, 4);
        scene.add(light);
    }
    {
        const color = 0xFFFFFF;
        const intensity = 1;
        const light = new THREE.DirectionalLight(color, intensity);
        light.position.set(1, -2, -4);
        scene.add(light);
    }
}

function onKeyDown(evt) {
    console.log("keydown", evt.key);
}

function onMouseMove(evt) {
    const pixelBuffer = picker.readMousePixels({
        canvas: canvas,
        renderer: renderer,
        evt: evt,
    });

    const coord = HexGridPicker.pickerIDToAxialCoord(pixelBuffer);
    if (coord) {
        const mesh = hexGridPicker.getHexMesh(coord);
        if (mesh) {
            if (hexGridPicker.currMesh) {
                hexGridPicker.currMesh.material.color.setHex(HEX_COLOR);
            }
            hexGridPicker.currMesh = mesh;

            mesh.material.color.setHex(SELECTED_HEX_COLOR);
        }
    }

}

function init() {
    battle = initial_battle();
    grid = battle.grid;

    /* from https://threejsfundamentals.org/threejs/lessons/threejs-fundamentals.html */
    canvas = document.getElementById('canvas');
    renderer = new THREE.WebGLRenderer({canvas});

    const rendererCtx = renderer.getContext();
    picker = new Picker({
        renderTarget: new THREE.WebGLRenderTarget(
            rendererCtx.drawingBufferWidth,
            rendererCtx.drawingBufferHeight,
        ),
        rootObject: new THREE.Object3D(),
        scene: new THREE.Scene(),
        debugCanvas: (debug
                      ? document.getElementById('picker-canvas')
                      : undefined),
    });

    const fov = 75;
    const aspect = canvas.width / canvas.height;
    const near = 0.1;
    const far = 100;

    camera = new THREE.PerspectiveCamera(fov, aspect, near, far);

    camera.position.z = 12;

    controls = new OrbitControls(camera, renderer.domElement);
    controls.screenSpacePanning = true;

	scene.background = new THREE.Color(0xeeeeee);

    const radius = 1;
    const geometry = hexShape(radius);
    const coords = JSON.parse(grid.get_coords_json());
    for (let i = 0; i < coords.length; i++) {
        addHexGeometry(geometry, coords[i], radius);
    }

    addLights(scene);
    scene.add(rootObject);
    renderer.render(scene, camera);

    /* only render the picker.scene as needed for getting clicks */
    picker.render({
        renderer: renderer,
        camera: camera,
    });

    window.addEventListener("mousemove", onMouseMove);
    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", onWindowResize, false);
}

function render() {
    controls.update();
    renderer.render(scene, camera);
    /* XXX: figure out how to hook onto what OrbitControls does */
    picker.render({
        renderer: renderer,
        camera: camera,
        sourceRootObject: rootObject,
    });
    requestAnimationFrame(render);
}

console.log(THREE.REVISION);
init();
requestAnimationFrame(render);
