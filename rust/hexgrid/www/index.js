import { initial_grid } from "hexgrid";
import * as THREE from "three";

const SQRT3 = Math.sqrt(3);
const HALF_SQRT3 = SQRT3 / 2;

var camera;
var canvas;
var grid;
var renderer;
var rootObject;
var scene;

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

/* convert to three.js coords */
function axialToPoint(coord, radius) {
    const {q, r} = coord;
    return {
        x: radius * ((SQRT3 * q) + (HALF_SQRT3 * r)),
        y: radius * r * 3 / 2,
    }
}

function onKeyDown(evt) {
    console.log("keydown", evt.key);
    switch (evt.key) {
    case "w":
        rootObject.rotation.x += 0.2;
        break;
    case "a":
        rootObject.rotation.z += 0.2;
        break;
    case "s":
        rootObject.rotation.x -= 0.2;
        break;
    case "d":
        rootObject.rotation.z -= 0.2;
        break;
    }
}

function init() {
    /* from https://threejsfundamentals.org/threejs/lessons/threejs-fundamentals.html */
    canvas = document.getElementById('canvas');
    grid = initial_grid();
    renderer = new THREE.WebGLRenderer({canvas});
    rootObject = new THREE.Object3D();
    scene = new THREE.Scene();

    const fov = 75;
    const aspect = canvas.width / canvas.height;
    const near = 0.1;
    const far = 100;

    camera = new THREE.PerspectiveCamera(fov, aspect, near, far);

    camera.position.z = 12;

	scene.background = new THREE.Color(0xeeeeee);

	rootObject.add(new THREE.HemisphereLight());

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


    const coords = JSON.parse(grid.get_coords_json());
    for (let i = 0; i < coords.length; i++) {
        const coord = coords[i];
        const radius = 1;
        const geometry = hexShape(radius);

        const point = axialToPoint(coord, radius + 0.1);
        console.log(coord, point);
        geometry.translate(point.x, point.y, 0);

        // const material = new THREE.MeshBasicMaterial({color: 0xaa00ff});
        const material = new THREE.MeshPhongMaterial({
            side: THREE.DoubleSide,
        });

        const hue = 0.55;
        const saturation = 0.7;
        const luminance = 0.1;
        material.color.setHSL(hue, saturation, luminance);

        const mesh = new THREE.Mesh(geometry, material);

        rootObject.add(mesh);
    }

    scene.add(rootObject);
    renderer.render(scene, camera);

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("resize", onWindowResize, false);
}

function render() {
    renderer.render(scene, camera);
    requestAnimationFrame(render);
}


console.log(THREE.REVISION);
init();
requestAnimationFrame(render);
