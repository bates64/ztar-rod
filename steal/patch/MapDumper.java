import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.ObjectInputStream;
import java.io.PrintWriter;

import editor.map.Map;
import editor.map.MutablePoint;
import editor.map.mesh.TexturedMesh;
import editor.map.mesh.Triangle;
import editor.map.mesh.Vertex;
import editor.map.shape.Model;
import editor.map.tree.MapObjectTreeModel;

public class MapDumper {

    public static void main(String[] args) throws ClassNotFoundException, IOException {
        File[] maps = new File("map/src").listFiles();

        for (int i = 0; i < maps.length; i++) {
            File file = maps[i];

            String name = file.getName();
            if (!name.endsWith(".map")) {
                continue;
            }

            System.out.print("\r" + i + " / " + maps.length + " : " + name + "                ");

            FileInputStream file_s = new FileInputStream(file);
            ObjectInputStream obj_s = new ObjectInputStream(file_s);
            Map map = (Map) obj_s.readObject();

            String outName = name.substring(0, name.length() - 3) + "json";
            PrintWriter out = new PrintWriter("map/src/" + outName, "UTF-8");
            MapDumper dumper = new MapDumper(out);
            dumper.dump(map);
        }

        System.out.println("\ndone");
    }


    private PrintWriter out;

    MapDumper(PrintWriter stream) {
        out = stream;
    }

    public void dump(Map map) throws IOException {
        C c = new C("{");
        c.item("\"bg_name\":\"" + map.bgName + "\"");
        c.item("\"meshes\":");

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

        c.end("}");

        out.flush();
    }

    public void printTriangles(Iterable<Triangle> ts) throws IOException {
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

    public void printVertex(Vertex v) throws IOException {
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
        o.item("\"uv\":");
            C uv = new C("[");
            uv.item(v.uv.getU());
            uv.item(v.uv.getV());
            uv.end("]");
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

    private class C {
        private boolean hasItem;

        C(String start) throws IOException {
            out.write(start);
            hasItem = false;
        }

        public void item() throws IOException {
            if (hasItem) out.write(",");
            else hasItem = true;
        }
        public void item(int i) throws IOException { item("" + i); }
        public void item(String e) throws IOException { item(); out.write(e); }

        public void end(String end) throws IOException {
            out.write(end);
        }
    }

}
