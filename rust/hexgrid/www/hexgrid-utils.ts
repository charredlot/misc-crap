import * as THREE from "three";

export interface AxialCoord {
    q: number,
    r: number,
}

export class HexGridPicker {
    meshByCoord: {[key: number]: {[key: number]: THREE.Mesh}};
    currMesh?: THREE.Mesh;

    constructor() {
        this.meshByCoord = {};
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

    static axialCoordToPickerID(coord: AxialCoord): THREE.Color {
        /*
         * have 3 bytes to work with for a THREE.Color
         * 0xqqqrrr
         * so store them as two 12-bit signed integers
         * also all the other colors in the scene are zero, so we set the sign
         * bit for zero to disambiguate
         */
        const {q, r} = coord;
        const absQ = Math.abs(q);
        const absR = Math.abs(r);

        console.assert(absQ <= 0x7ff, "q coord too big", coord);
        console.assert(absR <= 0x7ff, "r coord too big", coord);

        let id = (absQ << 12) + absR;
        if (q <= 0) {
            id = id | (1 << 23);
        }
        if (r <= 0) {
            id = id | (1 << 11);
        }
        return new THREE.Color(id);
    }

    static pickerIDToAxialCoord(pixelBuffer: Uint8Array): AxialCoord {
        /* all zeroes means nothing got clicked on */
        if ((pixelBuffer[0] === 0) &&
            (pixelBuffer[1] === 0) &&
            (pixelBuffer[2] === 0)) {
            return null;
        }

        /* 0xqqqrrr is in first 3 bytes, seems big endian? */
        let q = ((pixelBuffer[0] << 4) & 0x7f) | ((pixelBuffer[1] >> 4) & 0xf);
        let r = ((pixelBuffer[1] & 0x7) << 8) | pixelBuffer[2];

        /* sign bit is at the top of qqq */
        if ((q !== 0) && ((pixelBuffer[0] & 0x80) !== 0)) {
            q = q * -1;
        }
        /* sign bit is at the top of rrr */
        if ((r !== 0) && ((pixelBuffer[1] & 0x8) !== 0)) {
            r = r * -1;
        }
        return {q: q, r: r};
    }
}
