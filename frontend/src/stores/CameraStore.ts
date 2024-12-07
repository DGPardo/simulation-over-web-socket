export class CameraStore {
    center: { x: number; y: number };  // in screen coordinates
    zoom = 1;
    width: number;
    height: number;

    constructor(canvasWidth: number, canvasHeight: number) {
        this.width = canvasWidth;
        this.height = canvasHeight;
        this.center = { x: canvasHeight / 2, y: canvasWidth / 2 };
    }

    setCenter(x: number, y: number) {
        this.center = { x, y };
    }

    setZoom(zoom: number) {
        this.zoom = zoom;
    }

    screenToWorld(x: number, y: number): [number, number] {
        return [
            (x - this.width / 2) / this.zoom + this.center.x,
            (y - this.height / 2) / this.zoom + this.center.y,
        ];
    }

    worldToScreen(x: number, y: number): [number, number] {
        return [
            (x - this.center.x) * this.zoom + this.width / 2,
            (y - this.center.y) * this.zoom + this.height / 2,
        ];
    }

    zoomAt(x: number, y: number, deltaZoom: number) {
        this.setZoom(this.zoom * deltaZoom);
    }

    pan(dx: number, dy: number) {
        this.setCenter(this.center.x - dx / this.zoom, this.center.y - dy / this.zoom);
    }

    reset() {
        this.setCenter(0, 0);
        this.setZoom(1);
    }

    resize(width: number, height: number) {
        this.width = width;
        this.height = height;
    }
}
