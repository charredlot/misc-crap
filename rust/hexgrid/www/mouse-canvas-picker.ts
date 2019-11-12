import * as THREE from "three";

export interface PickerProps {
    renderTarget: THREE.WebGLRenderTarget,
    rootObject: THREE.Object3D,
    scene: THREE.Scene,
    debugCanvas?: HTMLCanvasElement,
}

/* XXX: put this elsewhere*/
export interface AxialCoord {
    q: number,
    r: number,
}

/*
 * this approach cribbed from threejs fundamentals
 * https://threejsfundamentals.org/threejs/lessons/threejs-picking.html
 */
export class Picker implements PickerProps {
    renderTarget: THREE.WebGLRenderTarget;
    rootObject: THREE.Object3D;
    scene: THREE.Scene;
    pixelBuffer: Uint8Array;

    meshByCoord: {[key: number]: {[key: number]: THREE.Mesh}};
    currMesh?: THREE.Mesh;

    debugCanvas?: HTMLCanvasElement;
    debugRenderer?: THREE.WebGLRenderer;

    constructor(props: PickerProps) {
        this.renderTarget = props.renderTarget;
        this.rootObject = props.rootObject;
        this.scene = props.scene;

        this.pixelBuffer = new Uint8Array(4);

        this.meshByCoord = {};

        this.scene.background = new THREE.Color(0);
        this.scene.add(this.rootObject);

        this.debugCanvas = props.debugCanvas;
        if (this.debugCanvas) {
            this.debugCanvas.style.display = "block";
            this.debugRenderer = new THREE.WebGLRenderer(
                {canvas: this.debugCanvas},
            );
        }
    }

    addHexMesh({q, r}: AxialCoord, mesh: THREE.Mesh): void {
        let byR = this.meshByCoord[q];
        if (!byR) {
            byR = {};
            byR[r] = mesh;
            this.meshByCoord[q] = byR;
        }
        else {
            byR[r] = mesh;
        }
    }

    getHexMesh({q, r}: AxialCoord): THREE.Mesh | null {
        const byR: any = this.meshByCoord[q];
        if (!byR) {
            return null;
        }

        return byR[r];
    }

    /* XXX: not sure when/why to use a different renderer vs setting target */
    render({renderer, camera, sourceRootObject}: {
        renderer: THREE.WebGLRenderer,
        camera: THREE.Camera,
        sourceRootObject?: THREE.Object3D,
    }): void {
        if (sourceRootObject) {
            this.rootObject.position.copy(sourceRootObject.position);
            this.rootObject.rotation.copy(sourceRootObject.rotation);
            this.rootObject.scale.copy(sourceRootObject.scale);
        }

        if (this.debugRenderer) {
            this.debugRenderer.render(this.scene, camera);
        }

        /* render the picker version of the scene */
        renderer.setRenderTarget(this.renderTarget);
        renderer.render(this.scene, camera);

        /* restore original */
        renderer.setRenderTarget(null);
    }

    readMousePixels({renderer, canvas, evt}: {
        renderer: THREE.WebGLRenderer,
        canvas: HTMLCanvasElement,
        evt: MouseEvent,
    }): Uint8Array {
        /* XXX: need to cache this rect? maybe only changes on resize? */
        const rect = canvas.getBoundingClientRect();
        const rendererCtx = renderer.getContext();

        /* readRenderTargetPixels is from the lower left so y is flipped */
        const x = evt.clientX - rect.left;
        const y = rendererCtx.drawingBufferHeight - (evt.clientY - rect.top);

        /* render the picker scene to figure out what got clicked */
        const pixelRatio = renderer.getPixelRatio();

        renderer.readRenderTargetPixels(
            this.renderTarget,
            x * pixelRatio, /* because of camera.setViewOffset, x offset is 0 */
            y * pixelRatio, /* ditto for y offset */
            1,
            1,
            this.pixelBuffer,
        );

        return this.pixelBuffer;
    }
}
