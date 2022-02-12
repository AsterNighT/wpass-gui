using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Configuration;
using System.Data;
using System.Drawing;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace wpass_gui
{
    public partial class FormSetting : Form
    {
        public FormSetting()
        {
            InitializeComponent();
        }

        private void formSetting_Load(object sender, EventArgs e)
        {
            var settings =
            ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None).AppSettings.Settings;
            if (settings[Constant.WPassPath] == null)
            {
                return;
            }
            textBoxPath.Text = settings[Constant.WPassPath].Value;
        }

        private void buttonSave_Click(object sender, EventArgs e)
        {
            var settings =
            ConfigurationManager.OpenExeConfiguration(ConfigurationUserLevel.None);
            if (settings.AppSettings.Settings[Constant.WPassPath] == null)
            {
                settings.AppSettings.Settings.Add(Constant.WPassPath, textBoxPath.Text);
            }
            else
            {
                settings.AppSettings.Settings.Remove(Constant.WPassPath);
                settings.AppSettings.Settings.Add(Constant.WPassPath, textBoxPath.Text);
            }
            settings.Save(ConfigurationSaveMode.Modified);
            ConfigurationManager.RefreshSection("appSettings");
            this.Close();
        }
    }
}
