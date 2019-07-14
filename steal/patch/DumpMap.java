import java.io.FileInputStream;
import java.io.IOException;
import java.io.ObjectInputStream;

import editor.map.Map;
import editor.map.MutablePoint;
import editor.map.mesh.TexturedMesh;
import editor.map.mesh.Triangle;
import editor.map.mesh.Vertex;
import editor.map.shape.Model;
import editor.map.tree.MapObjectTreeModel;

public class DumpMap {

    public static void main(String[] args) throws ClassNotFoundException, IOException {
        FileInputStream file_s = new FileInputStream(args[0]);
        ObjectInputStream obj_s = new ObjectInputStream(file_s);
        Map map = (Map) obj_s.readObject();

        MapObjectTreeModel<Model> modelTree = map.modelTree;

        C models = new C("[");

        for (Model model : modelTree) {
            TexturedMesh mesh = model.getMesh();
            if (mesh == null) continue;

            models.item();

            C m = new C("{");
            m.item("\"texture\":\"" + mesh.textureName + "\"");
            m.item("\"triangles\":"); printTriangles(mesh);
            m.end("}");
        }

        models.end("]");
    }

    public static void printTriangles(Iterable<Triangle> ts) {
        C c = new C("[");

        for (Triangle t : ts) {
            c.item();

            C v = new C("[");
            v.item(); printVertex(t.vert[0]);
            v.item(); printVertex(t.vert[1]);
            v.item(); printVertex(t.vert[2]);
            v.end("]");
        }

        c.end("]");
    }

    public static void printVertex(Vertex v) {
        v.useLocal = false;

        C o = new C("{");
        o.item("\"xyz\":");
            C xyz = new C("[");
            xyz.item(v.getCurrentX());
            xyz.item(v.getCurrentY());
            xyz.item(v.getCurrentZ());
            xyz.end("]");
        o.item("\"rgba\":");
            C rgba = new C("[");
            rgba.item(asUnsigned(v.r));
            rgba.item(asUnsigned(v.g));
            rgba.item(asUnsigned(v.b));
            rgba.item(asUnsigned(v.a));
            rgba.end("]");
        o.end("}");
    }

    public static int asUnsigned(byte b) {
        int signed = b;
        return b & 0xFF;
    }


    // Utility class for creating possibly empty objects and arrays.
    // "C" is short for "container".
    //
    // Usage:
    //     C c = new C("[");
    //     for (...) {
    //         c.item("3");
    //     }
    //     c.end("]");

    private static class C {
        private boolean hasItem;

        C(String start) {
            System.out.print(start);
            hasItem = false;
        }

        public void item() {
            if (hasItem) System.out.print(",");
            else hasItem = true;
        }
        public void item(int i) { item(); System.out.print(i); }
        public void item(String e) { item(); System.out.print(e); }

        public void end(String end) {
            System.out.print(end);
        }
    }

}
